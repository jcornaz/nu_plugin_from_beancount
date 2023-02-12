use beancount_parser::{
    transaction::{Flag, Posting},
    Amount, Date, Transaction,
};
use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone};
use nu_protocol::{Span, Value};

pub(crate) fn record(trx: &Transaction<'_>, span: Span) -> Value {
    Value::record(
        vec![
            "date".into(),
            "directive".into(),
            "flag".into(),
            "payee".into(),
            "narration".into(),
            "postings".into(),
        ],
        vec![
            date(trx.date(), span),
            Value::string("txn", span),
            flag(trx.flag(), span),
            trx.payee()
                .map(|n| Value::string(n, span))
                .unwrap_or_default(),
            trx.narration()
                .map(|d| Value::string(d, span))
                .unwrap_or_default(),
            Value::list(
                trx.postings().iter().map(|p| posting(p, span)).collect(),
                span,
            ),
        ],
        span,
    )
}

fn posting(posting: &Posting<'_>, span: Span) -> Value {
    Value::record(
        vec!["account".into(), "amount".into()],
        vec![
            Value::string(posting.account().to_string(), span),
            posting
                .amount()
                .map(|a| amount(a, span))
                .unwrap_or_default(),
        ],
        span,
    )
}

pub(super) fn amount(amount: &Amount<'_>, span: Span) -> Value {
    Value::record(
        vec![amount.currency().into()],
        vec![amount
            .value()
            .try_into_f64()
            .map(|v| Value::float(v, span))
            .unwrap_or_default()],
        span,
    )
}

fn flag(flag: Option<Flag>, span: Span) -> Value {
    Value::string(
        match flag {
            Some(Flag::Cleared) | None => "*",
            Some(Flag::Pending) => "!",
        },
        span,
    )
}

pub(super) fn date(date: Date, span: Span) -> Value {
    let naive = NaiveDate::from_ymd_opt(
        date.year().into(),
        date.month_of_year().into(),
        date.day_of_month().into(),
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
    use approx::assert_ulps_eq;
    use beancount_parser::{Directive, Parser};
    use chrono::NaiveDate;

    use super::*;

    fn input_trx(raw: &str) -> Transaction<'_> {
        Parser::new(raw)
            .filter_map(Result::ok)
            .find_map(Directive::into_transaction)
            .unwrap()
    }

    fn parse(raw: &str) -> Value {
        record(&input_trx(raw), Span::unknown())
    }

    #[rstest]
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
    fn should_return_expected_transaction_flag(#[case] input: &str, #[case] expected_flag: &str) {
        let trx = parse(input);
        assert_eq!(
            trx.get_data_by_key("flag").unwrap().as_string().unwrap(),
            expected_flag
        );
    }

    #[rstest]
    fn should_return_date(#[values(r#"2022-02-05 txn"#)] input: &str) {
        let trx = parse(input);
        let Value::Date { val, .. } = &trx.get_data_by_key("date").unwrap() else { 
            panic!("was not a date");
        };
        let expected = NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(val.date_naive(), expected);
    }

    #[rstest]
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
