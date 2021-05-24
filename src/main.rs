#![forbid(unsafe_code)]

use std::{env, fs::File, io};

use anyhow::{bail, Result};
use transaction_oxidizer::{output_clients, process_transactions};

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        bail!("Usage: transaction-oxidizer <input_file>");
    }
    let mut input_file = File::open(&args[1])?;
    let clients = process_transactions(&mut input_file)?;
    let mut output_file = io::stdout();
    output_clients(&mut output_file, clients.values())?;
    Ok(())
}
