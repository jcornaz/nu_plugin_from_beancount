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
use nu_protocol::{Category, Signature, Spanned, Value};

use nu_plugin::{serve_plugin, MsgPackSerializer};

fn main() {
    serve_plugin(&mut NuPlugin, MsgPackSerializer {});
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
        let mut transactions = Vec::new();
        for directive in Parser::new(&input) {
            match directive {
                Err(err) => {
                    return Err(LabeledError {
                        label: "Invalid beancount input".into(),
                        msg: format!("Error while parsing beancount file: {err:?}"),
                        span: Some(input_span),
                    });
                }
                Ok(Directive::Transaction(trx)) => {
                    transactions.push(transaction::record(&trx, input_span));
                }
                Ok(_) => (),
            }
        }
        Ok(Value::record(
            vec!["transactions".into()],
            vec![Value::list(transactions, input_span)],
            call.head,
        ))
    }
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
        let signature = plugin
            .signature()
            .into_iter()
            .find(|s| &s.name == "from beancount")
            .unwrap();
        assert_eq!(signature.category, Category::Formats);
    }

    #[rstest]
    fn should_be_successful(#[values("", BEAN_EXAMPLE)] input: &str) {
        let result = from_beancount(input);
        assert!(result.is_ok(), "{result:?}");
    }

    #[rstest]
    fn should_return_empty_list_of_trx_for_empty_input() {
        let transactions = from_beancount("")
            .unwrap()
            .get_data_by_key("transactions")
            .unwrap();
        assert!(transactions.as_list().unwrap().is_empty());
    }

    #[rstest]
    fn should_return_transaction() {
        let input = r#"
2022-02-05 * "Groceries Store" "Groceries"
    Expenses:Food    10 CHF
    Assets:Cash
        "#;
        let output = from_beancount(input).unwrap();
        let transactions_value = output.get_data_by_key("transactions").unwrap();
        let transactions = transactions_value.as_list().unwrap();
        assert_eq!(transactions.len(), 1);
        let directive = &transactions[0];
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
    fn should_return_transaction_date(#[values(r#"2022-02-05 txn"#)] input: &str) {
        let output = from_beancount(input).unwrap();
        let transactions_value = output.get_data_by_key("transactions").unwrap();
        let transactions = transactions_value.as_list().unwrap();
        assert_eq!(transactions.len(), 1);
        let Value::Date { val, .. } = &transactions[0].get_data_by_key("date").unwrap() else { 
            panic!("was not a date");
        };
        let expected = NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(val.date_naive(), expected);
    }

    fn from_beancount(input: &str) -> Result<Value, LabeledError> {
        plugin().run("from beancount", &SIMPLE_CALL, &Value::test_string(input))
    }
}
