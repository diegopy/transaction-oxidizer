use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Read},
    path::PathBuf,
};
use transaction_oxidizer::{output_clients, process_transactions};

#[test]
fn test_samples() -> Result<()> {
    let input_file_names = (1..).map(|n| PathBuf::from(&format!("tests/data/input_{}.csv", n)));
    let output_file_names = (1..).map(|n| PathBuf::from(&format!("tests/data/output_{}.csv", n)));
    for (input_file_name, output_file_name) in input_file_names.zip(output_file_names) {
        match File::open(&input_file_name) {
            Ok(mut input_file) => {
                let mut output_file = File::open(&output_file_name).with_context(|| {
                    format!("opening output file {}", output_file_name.display())
                })?;
                let verify_result = verify_output(&mut input_file, &mut output_file)
                    .with_context(|| format!("procesing {}", input_file_name.display()))?;
                assert!(
                    matches!(verify_result, VerifyResult::Match),
                    "verify output for {} failed {:?}",
                    output_file_name.display(),
                    verify_result
                );
            }
            Err(error) => {
                if let ErrorKind::NotFound = error.kind() {
                    break;
                } else {
                    return Err(error.into());
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
enum VerifyResult {
    Match,
    Different {
        line: usize,
        generated_line: String,
        sample_line: String,
    },
}

fn verify_output(
    sample_input: &mut impl Read,
    sample_output: &mut impl Read,
) -> Result<VerifyResult> {
    let clients = process_transactions(sample_input)?;
    let mut clients = clients.values().collect::<Vec<_>>();
    clients.sort_by(|a, b| a.id.cmp(&b.id));
    let mut clients_output = vec![];
    output_clients(&mut clients_output, clients.into_iter())?;
    let mut clients_reader = BufReader::new(clients_output.as_slice());
    let mut output_reader = BufReader::new(sample_output);
    let mut buf = String::new();
    let mut output_buf = String::new();
    let mut line = 1usize;
    let match_result = loop {
        let generated_bytes_read = clients_reader.read_line(&mut buf)?;
        let sample_bytes_read = output_reader.read_line(&mut output_buf)?;
        if generated_bytes_read == 0 && sample_bytes_read == 0 {
            break VerifyResult::Match;
        }
        if buf != output_buf {
            break VerifyResult::Different {
                line,
                generated_line: buf,
                sample_line: output_buf,
            };
        }
        line += 1;
        buf.clear();
        output_buf.clear();
    };
    Ok(match_result)
}
