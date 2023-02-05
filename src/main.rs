#[cfg(test)]
#[macro_use]
extern crate rstest;

use std::collections::HashMap;

use beancount_parser::{Directive, Parser};
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

mod field {
    pub(super) const DIRECTIVE_TYPE: &str = "directive_type";
    pub(super) const PAYEE: &str = "payee";
    pub(super) const NARRATION: &str = "narration";
}

fn into_record(directive: Directive<'_>, span: Span) -> Option<Value> {
    let mut map = HashMap::with_capacity(1);
    if let Directive::Transaction(trx) = directive {
        map.insert(field::DIRECTIVE_TYPE.into(), Value::string("txn", span));
        map.insert(
            field::PAYEE.into(),
            trx.payee()
                .map(|n| Value::string(n, span))
                .unwrap_or_default(),
        );
        map.insert(
            field::NARRATION.into(),
            trx.narration()
                .map(|d| Value::string(d, span))
                .unwrap_or_default(),
        );
        Some(Value::record_from_hashmap(&map, span))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
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

    fn from_beancount(input: &str) -> Result<Value, LabeledError> {
        plugin().run("from beancount", &SIMPLE_CALL, &Value::test_string(input))
    }
}
