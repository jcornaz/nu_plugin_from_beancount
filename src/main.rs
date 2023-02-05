#[cfg(test)]
#[macro_use]
extern crate rstest;

use beancount_parser::{Date, Directive, Parser};
use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone};
use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Category, Signature, Span, Spanned, Value};

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&mut NuPlugin, MsgPackSerializer {})
}

pub struct NuPlugin;

impl nu_plugin::Plugin for NuPlugin {
    fn signature(&self) -> Vec<Signature> {
        vec![Signature::build("from beancount")
            .usage("Convert from beancount to structured data")
            .category(Category::Formats)]
    }

    fn run(
        &mut self,
        _name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let Spanned {
            item: input,
            span: input_span,
        } = input.as_spanned_string()?;
        let vals = Parser::new(&input)
            .filter_map(|directive| match directive {
                Ok(directive) => into_record(directive, input_span).map(Ok),
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<Value>, _>>()
            .map_err(|err| LabeledError {
                label: "Invalid beancount input".into(),
                msg: format!("Error while parsing beancount file: {err:?}"),
                span: Some(input_span),
            })?;
        Ok(Value::List {
            vals,
            span: call.head,
        })
    }
}

fn into_record(directive: Directive<'_>, span: Span) -> Option<Value> {
    if let Directive::Transaction(trx) = directive {
        Some(Value::record(
            vec![
                "directive_type".into(),
                "date".into(),
                "payee".into(),
                "narration".into(),
            ],
            vec![
                Value::string("txn", span),
                into_date(trx.date(), span),
                trx.payee()
                    .map(|n| Value::string(n, span))
                    .unwrap_or_default(),
                trx.narration()
                    .map(|d| Value::string(d, span))
                    .unwrap_or_default(),
            ],
            span,
        ))
    } else {
        None
    }
}

fn into_date(date: Date, span: Span) -> Value {
    let naive = NaiveDate::from_ymd_opt(
        date.year() as i32,
        date.month_of_year() as u32,
        date.day_of_month() as u32,
    )
    .expect("The date given by the beancount-parser should be valid")
    .and_time(NaiveTime::default());

    let val = FixedOffset::east_opt(0)
        .unwrap()
        .from_local_datetime(&naive)
        .unwrap();

    Value::Date { val, span }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use nu_plugin::Plugin;
    use nu_protocol::Span;

    use super::*;

    const SIMPLE_CALL: EvaluatedCall = EvaluatedCall {
        head: Span::unknown(),
        positional: Vec::new(),
        named: Vec::new(),
    };

    const BEAN_EXAMPLE: &str = include_str!("example.beancount");

    #[fixture]
    fn plugin() -> impl Plugin {
        NuPlugin
    }

    #[rstest]
    fn should_have_from_beancount_command(plugin: impl Plugin) {
        assert!(plugin
            .signature()
            .into_iter()
            .any(|s| &s.name == "from beancount"));
    }

    #[rstest]
    fn should_be_successful(#[values("", BEAN_EXAMPLE)] input: &str) {
        let result = from_beancount(input);
        assert!(result.is_ok(), "{result:?}");
    }

    #[rstest]
    fn all_row_should_have_directive_type(#[values("", BEAN_EXAMPLE)] input: &str) {
        let output = from_beancount(input).unwrap();
        assert!(output
            .as_list()
            .unwrap()
            .iter()
            .all(|row| { row.get_data_by_key("directive_type").is_some() }));
    }

    #[rstest]
    fn should_return_empty_list_for_empty_input() {
        let result = from_beancount("");
        let Ok(Value::List { vals, .. }) = result else {
            panic!("Expected a list value but was: {result:?}");
        };
        assert!(vals.is_empty());
    }

    #[rstest]
    fn should_return_transaction() {
        let input = r#"
2022-02-05 * "Groceries Store" "Groceries"
    Expenses:Food    10 CHF
    Assets:Cash
        "#;
        let output = from_beancount(input).unwrap();
        let directives = output.as_list().unwrap();
        assert_eq!(directives.len(), 1);
        let directive = &directives[0];
        assert_eq!(
            directive
                .get_data_by_key("directive_type")
                .unwrap()
                .as_string()
                .unwrap(),
            "txn"
        );
        assert_eq!(
            directive
                .get_data_by_key("payee")
                .unwrap()
                .as_string()
                .unwrap(),
            "Groceries Store"
        );
        assert_eq!(
            directive
                .get_data_by_key("narration")
                .unwrap()
                .as_string()
                .unwrap(),
            "Groceries"
        );
    }

    #[rstest]
    fn should_return_date(#[values(r#"2022-02-05 txn"#)] input: &str) {
        let output = from_beancount(input).unwrap();
        let directives = output.as_list().unwrap();
        assert_eq!(directives.len(), 1);
        let Value::Date { val, .. } = &directives[0].get_data_by_key("date").unwrap() else { 
            panic!("was not a date");
        };
        let expected = NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(val.date_naive(), expected);
    }

    fn from_beancount(input: &str) -> Result<Value, LabeledError> {
        plugin().run("from beancount", &SIMPLE_CALL, &Value::test_string(input))
    }
}
