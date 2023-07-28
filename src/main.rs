#![deny(
    future_incompatible,
    nonstandard_style,
    unsafe_code,
    private_in_public,
    unused_results
)]
#![warn(rust_2018_idioms, clippy::pedantic)]
#![cfg_attr(test, allow(clippy::needless_pass_by_value))]

#[cfg(test)]
#[macro_use]
extern crate rstest;

mod transaction;

use beancount_parser::{Directive, Parser};
use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Category, PluginSignature, Span, Spanned, Value};

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&mut NuPlugin, MsgPackSerializer {});
}

pub struct NuPlugin;

impl nu_plugin::Plugin for NuPlugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("from beancount")
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
                Ok(directive) => record(&directive, input_span).map(Ok),
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

pub(crate) fn record(directive: &Directive<'_>, span: Span) -> Option<Value> {
    match directive {
        Directive::Transaction(trx) => Some(transaction::record(trx, span)),
        Directive::Include(include) => Some(Value::record(
            vec!["directive".into(), "path".into()],
            vec![
                Value::string("include", span),
                Value::string(include.path().to_str().unwrap(), span),
            ],
            span,
        )),
        Directive::Assertion(balance) => Some(Value::record(
            vec![
                "directive".into(),
                "date".into(),
                "account".into(),
                "amount".into(),
            ],
            vec![
                Value::string("balance", span),
                transaction::date(balance.date(), span),
                Value::string(balance.account().to_string(), span),
                transaction::amount(balance.amount(), span),
            ],
            span,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use approx::assert_ulps_eq;
    use chrono::NaiveDate;
    use nu_plugin::Plugin;

    use super::*;

    const SIMPLE_CALL: EvaluatedCall = EvaluatedCall {
        head: Span::unknown(),
        positional: Vec::new(),
        named: Vec::new(),
    };

    const BEAN_EXAMPLE: &str = include_str!("../example.beancount");

    #[fixture]
    fn plugin() -> impl Plugin {
        NuPlugin
    }

    #[rstest]
    fn should_have_from_beancount_command(plugin: impl Plugin) {
        let signature = plugin
            .signature()
            .into_iter()
            .find(|s| &s.sig.name == "from beancount")
            .unwrap();
        assert_eq!(signature.sig.category, Category::Formats);
    }

    #[rstest]
    fn should_be_successful(#[values("", BEAN_EXAMPLE)] input: &str) {
        let result = from_beancount(input);
        assert!(result.is_ok(), "{result:?}");
    }

    #[rstest]
    fn all_row_should_have_directive_type(#[values("", BEAN_EXAMPLE)] input: &str) {
        let output = from_beancount_success(input);
        assert!(output
            .iter()
            .all(|row| { row.get_data_by_key("directive").is_some() }));
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
        let directives = from_beancount_success(input);
        assert_eq!(directives.len(), 1);
        let directive = &directives[0];
        assert_eq!(
            directive
                .get_data_by_key("directive")
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
        let directives = from_beancount_success(input);
        assert_eq!(directives.len(), 1);
        let Value::Date { val, .. } = &directives[0].get_data_by_key("date").unwrap() else { 
            panic!("was not a date");
        };
        let expected = NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(val.date_naive(), expected);
    }

    #[test]
    fn should_return_include_directives() {
        let input = r#"include "path/to/file.beancount""#;
        let directives = from_beancount_success(input);
        assert_eq!(directives.len(), 1);
        assert_eq!(
            &directives[0]
                .get_data_by_key("directive")
                .expect("no 'directive' field")
                .as_string()
                .expect("'directive' is not a string"),
            "include"
        );
        assert_eq!(
            directives[0]
                .get_data_by_key("path")
                .expect("no 'path' field")
                .as_path()
                .expect("'path' cannot be converted to path"),
            PathBuf::from("path/to/file.beancount"),
        );
    }

    #[test]
    fn should_return_balance_assertion() {
        let input = r"2014-12-26 balance Liabilities:US:CreditCard   -3492.02 USD";
        let directives = from_beancount_success(input);
        assert_eq!(directives.len(), 1);
        assert_eq!(
            &directives[0]
                .get_data_by_key("directive")
                .expect("no 'directive' field")
                .as_string()
                .expect("'directive' is not a string"),
            "balance"
        );
        let Some(Value::Date { val: date, .. }) = directives[0].get_data_by_key("date") else {
            panic!("Invalid or absent 'date'");
        };
        assert_eq!(
            date.date_naive(),
            NaiveDate::from_ymd_opt(2014, 12, 26).unwrap()
        );
        assert_eq!(
            &directives[0]
                .get_data_by_key("account")
                .expect("no 'account' field")
                .as_string()
                .expect("'account' is not a string"),
            "Liabilities:US:CreditCard",
        );
        let amount = &directives[0]
            .get_data_by_key("amount")
            .expect("no 'amount' field");
        assert_ulps_eq!(
            amount
                .get_data_by_key("USD")
                .expect("no 'USD' key in amount")
                .as_float()
                .expect("the amount value is not a number"),
            -3492.02
        );
    }

    fn from_beancount_success(input: &str) -> Vec<Value> {
        from_beancount(input)
            .expect("`from beancount` failed")
            .as_list()
            .expect("`from beancount` did not return a list")
            .to_owned()
    }

    fn from_beancount(input: &str) -> Result<Value, LabeledError> {
        plugin().run("from beancount", &SIMPLE_CALL, &Value::test_string(input))
    }
}
