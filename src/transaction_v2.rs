#![cfg(test)]

use std::collections::HashMap;

use beancount_parser as parser;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone};
use nu_protocol::{CustomValue, Span, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    date: DateTime<FixedOffset>,
    directive: String,
    flag: Flag,
    payee: Option<String>,
    narration: Option<String>,
    postings: Vec<Posting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Flag {
    #[serde(rename = "*")]
    Cleared,
    #[serde(rename = "!")]
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Posting {
    account: Account,
    amount: Option<Amount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Account(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Amount(HashMap<String, f64>);

#[typetag::serde]
impl CustomValue for Transaction {
    fn clone_value(&self, span: Span) -> Value {
        Value::CustomValue {
            val: Box::new(self.clone()),
            span,
        }
    }

    fn value_string(&self) -> String {
        self.typetag_name().into()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, nu_protocol::ShellError> {
        Ok(Value::nothing(span))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) fn record(trx: &parser::Transaction<'_>, span: Span) -> Value {
    Value::CustomValue {
        val: Box::new(Transaction {
            date: date(trx.date()),
            directive: "txn".into(),
            flag: trx.flag().into(),
            payee: trx.payee().map(ToOwned::to_owned),
            narration: trx.narration().map(ToOwned::to_owned),
            postings: trx.postings().iter().map(Into::into).collect(),
        }),
        span,
    }
}

impl From<&'_ parser::transaction::Posting<'_>> for Posting {
    fn from(value: &'_ parser::transaction::Posting<'_>) -> Self {
        Posting {
            account: Account(value.account().to_string()),
            amount: value.amount().map(Into::into),
        }
    }
}

impl From<&'_ parser::Amount<'_>> for Amount {
    fn from(amount: &'_ parser::Amount<'_>) -> Self {
        let mut map = HashMap::with_capacity(1);
        if let Ok(value) = amount.value().try_into_f64() {
            let _ = map.insert(amount.currency().to_owned(), value);
        }
        Amount(map)
    }
}

impl From<Option<parser::transaction::Flag>> for Flag {
    fn from(value: Option<parser::transaction::Flag>) -> Self {
        match value {
            Some(parser::transaction::Flag::Cleared) | None => Flag::Cleared,
            Some(parser::transaction::Flag::Pending) => Flag::Pending,
        }
    }
}

fn date(date: parser::Date) -> DateTime<FixedOffset> {
    let naive = NaiveDate::from_ymd_opt(
        date.year().into(),
        date.month_of_year().into(),
        date.day_of_month().into(),
    )
    .expect("The date given by the beancount-parser should be valid")
    .and_time(NaiveTime::default());

    FixedOffset::east_opt(0)
        .unwrap()
        .from_local_datetime(&naive)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use beancount_parser::{Directive, Parser};
    use chrono::NaiveDate;

    use super::*;

    fn input_trx(raw: &str) -> parser::Transaction<'_> {
        Parser::new(raw)
            .filter_map(Result::ok)
            .find_map(Directive::into_transaction)
            .unwrap()
    }

    fn parse(raw: &str) -> Value {
        record(&input_trx(raw), Span::unknown())
    }

    #[rstest]
    #[ignore]
    fn should_return_type_payee_and_narration() {
        let input = r#"2022-02-05 * "Groceries Store" "Groceries""#;
        let trx = parse(input);
        assert_eq!(
            trx.get_data_by_key("directive")
                .unwrap()
                .as_string()
                .unwrap(),
            "txn"
        );
        assert_eq!(
            trx.get_data_by_key("payee").unwrap().as_string().unwrap(),
            "Groceries Store"
        );
        assert_eq!(
            trx.get_data_by_key("narration")
                .unwrap()
                .as_string()
                .unwrap(),
            "Groceries"
        );
    }

    #[rstest]
    #[case("2022-02-05 *", "*")]
    #[case("2022-02-05 !", "!")]
    #[case("2022-02-05 txn", "*")]
    #[ignore]
    fn should_return_expected_transaction_flag(#[case] input: &str, #[case] expected_flag: &str) {
        let trx = parse(input);
        assert_eq!(
            trx.get_data_by_key("flag").unwrap().as_string().unwrap(),
            expected_flag
        );
    }

    #[rstest]
    #[ignore]
    fn should_return_date(#[values(r#"2022-02-05 txn"#)] input: &str) {
        let trx = parse(input);
        let Value::Date { val, .. } = &trx.get_data_by_key("date").unwrap() else { 
            panic!("was not a date");
        };
        let expected = NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(val.date_naive(), expected);
    }

    #[rstest]
    #[ignore]
    fn should_return_posings_in_transaction() {
        let input = r#"
2022-02-05 * "Groceries Store" "Groceries"
    Expenses:Food    10 CHF
    Assets:Cash
        "#;
        let trx = parse(input);
        let postings = trx.get_data_by_key("postings").unwrap();
        let posting_list = postings.as_list().unwrap();
        assert_eq!(posting_list.len(), 2);
        assert_eq!(
            &posting_list[0]
                .get_data_by_key("account")
                .unwrap()
                .as_string()
                .unwrap(),
            "Expenses:Food"
        );
        let food_amount = posting_list[0].get_data_by_key("amount").unwrap();
        let (cols, vals) = food_amount.as_record().unwrap();
        assert_eq!(cols, &["CHF".to_string()]);
        assert_ulps_eq!(vals[0].as_float().unwrap(), 10.0);
        assert_eq!(
            &posting_list[1]
                .get_data_by_key("account")
                .unwrap()
                .as_string()
                .unwrap(),
            "Assets:Cash"
        );
        let cash_amount = &posting_list[1].get_data_by_key("amount").unwrap();
        assert!(cash_amount.is_nothing());
    }
}
