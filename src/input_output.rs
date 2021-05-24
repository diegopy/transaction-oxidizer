use crate::{
    account::ClientAccount,
    transaction::{AmountTransaction, Transaction, TransactionData},
    Money,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TransactionRowType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionRow {
    #[serde(rename = "type")]
    pub transaction_type: TransactionRowType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

#[derive(Serialize)]
pub struct ClientRecord {
    pub client: u16,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

impl From<&ClientAccount> for ClientRecord {
    fn from(account: &ClientAccount) -> Self {
        Self {
            client: account.id,
            available: account.available.to_string(),
            held: account.held.to_string(),
            total: (account.held + account.available).to_string(),
            locked: account.locked,
        }
    }
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = TransactionConversionError;

    fn try_from(value: TransactionRow) -> Result<Self, Self::Error> {
        use Transaction::*;
        let tx_data = TransactionData {
            client: value.client,
            tx: value.tx,
        };
        let transaction = match value.transaction_type {
            TransactionRowType::Deposit => Deposit(AmountTransaction {
                tx_data,
                amount: value
                    .amount
                    .as_ref()
                    .ok_or_else(|| TransactionConversionError::MissingAmountData(value.clone()))
                    .and_then(|str_amount| {
                        Money::from_str(&str_amount)
                            .map_err(|err| TransactionConversionError::DecimalAmountConversion(err))
                    })?,
            }),
            TransactionRowType::Withdrawal => Withdrawal(AmountTransaction {
                tx_data,
                amount: value
                    .amount
                    .as_ref()
                    .ok_or_else(|| TransactionConversionError::MissingAmountData(value.clone()))
                    .and_then(|str_amount| {
                        Money::from_str(&str_amount)
                            .map_err(TransactionConversionError::DecimalAmountConversion)
                    })?,
            }),
            TransactionRowType::Dispute => Dispute(tx_data),
            TransactionRowType::Resolve => Resolve(tx_data),
            TransactionRowType::Chargeback => ChargeBack(tx_data),
        };
        Ok(transaction)
    }
}

#[derive(Error, Debug)]
pub enum TransactionConversionError {
    #[error("missing amount data in row {0:?}")]
    MissingAmountData(TransactionRow),

    #[error("error converting decimal amount")]
    DecimalAmountConversion(#[source] <Money as FromStr>::Err),
}

#[cfg(test)]
mod tests {

    use csv::{ReaderBuilder, Trim};

    use super::*;

    #[test]
    fn parses_ok() {
        let mut records = vec![];
        let input = "type, client, tx, amount\n\
                          deposit,    1, 1, 1.0\n\
                          deposit,    2, 2, 2.0\n\
                          deposit,    1, 3, 2.0\n\
                          withdrawal, 1, 4, 1.5\n\
                          withdrawal, 2, 5, 3.0\n\
                          dispute,    1, 2,";
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(input.as_bytes());
        for record in reader.deserialize() {
            let record: TransactionRow = record.expect("parsing record");
            records.push(record);
        }
        assert_eq!(6, records.len());
    }

    #[test]
    fn decimal_parsing() {
        let num = Money::from_str("11111111111111111111111111111111111.0001").unwrap();
        assert_eq!("11111111111111111111111111111111111.0001", num.to_string());
    }
}
