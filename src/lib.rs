#![forbid(unsafe_code)]

mod account;
#[cfg(not(feature = "rust_decimal"))]
mod fixed_point;
mod input_output;
mod transaction;

pub use account::ClientAccount;
use anyhow::{Context, Result};
use csv::{ReaderBuilder, Trim};
pub use input_output::{ClientRecord, TransactionRow};
#[cfg(feature = "rust_decimal")]
use rust_decimal::Decimal;
use std::{
    collections::HashMap,
    convert::TryInto,
    io::{Read, Write},
};
pub use transaction::Transaction;

#[cfg(feature = "rust_decimal")]
pub type Money = Decimal;

#[cfg(not(feature = "rust_decimal"))]
type Money = fixed_point::DecimalFixedPoint<i128, 4>;

pub fn output_clients<'a>(
    output_file: &mut impl Write,
    clients: impl Iterator<Item = &'a ClientAccount>,
) -> Result<()> {
    let mut writer = csv::Writer::from_writer(output_file);
    Ok(for client in clients {
        let client_record: ClientRecord = client.into();
        writer.serialize(client_record)?;
    })
}

pub fn process_transactions(input_file: &mut impl Read) -> Result<HashMap<u16, ClientAccount>> {
    let mut clients: HashMap<u16, ClientAccount> = HashMap::new();
    process_transactions_into_accounts(input_file, &mut clients)?;
    Ok(clients)
}

pub fn process_transactions_into_accounts(
    input_file: &mut impl Read,
    clients: &mut HashMap<u16, ClientAccount>,
) -> Result<()> {
    let mut line_number = 2; // Records start at line 2. 1 is for the headers.
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(input_file);
    Ok(for record in reader.deserialize() {
        let row: TransactionRow = record?;
        let client_id = row.client;
        let tx: Transaction = row
            .try_into()
            .with_context(|| format!("parsing line number {}", line_number))?;
        let client = clients
            .entry(client_id)
            .or_insert_with(|| ClientAccount::new(client_id));
        if let Err(error) = client.try_apply(tx) {
            log::info!("Error applying transaction: {}. Continuing.", error);
        }
        line_number += 1;
    })
}
