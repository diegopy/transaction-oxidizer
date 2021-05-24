# transaction-oxidizer
CSV transaction engine

## Requirements
- [Rust](https://www.rust-lang.org/).

## Build
- `cargo build --release` will build an optimized release build in the `target` directory.
## Configuration
- The `RUST_LOG` environment variable controls the logging level as explained [here](https://docs.rs/env_logger/0.8.3/env_logger/).

## Usage
- `cargo run -- tests/data/sample_1.csv > output.csv` will process the transactions from `sample_1.csv` and output the result to `stdout`, redirected to `output.csv` in this example.

## Architecture and design
As indicated in the requirements document special care was taken to ensure completeness, correctness, safety and robustness, efficiency and maintainability. The program was structured as a lib crate that handles transaction processing, accounts keeping, io and fixed point handling (explained below) with the main binary crate providing just the "user interface" CLI and argument parsing.

## Completeness and correctness
The code was tested by various unit tests and by an integration test using sample input and output files located under /tests/data. `cargo test` will run the test harness. It discovers the sample files by looking for input_{n}.csv and output_{n}.csv starting from n = 1. For each pair of files, it processes the transactions on the input file and compares the generated output against the corresponding output file. If any difference is found the test fails.
The sample files exercise all transaction types and their effects: deposit, withdrawal, dispute, resolve and charge-back. It also tests the correct handling of large monetary amounts.
Regarding monetary amounts as a design decision I considered that floating point numbers are not an appropriate data type as they can represent exactly decimal amounts. They also can lose precision when applying arithmetic operations. Because of this I implemented evaluated two crates that implement fixed-point functionality: [bigdecimal-rs](https://github.com/akubera/bigdecimal-rs) and [rust_decimal](https://github.com/paupino/rust-decimal). I discarded bigdecimal-rs due to lackluster maintenance as many issues on the repo remained unanswered by the maintainer. rust_decimal seemed like a more robust choice. After implementing it to handle the amount though I quickly discovered some bugs in handling of the parsing of big numbers, where it would truncate the result without any error. I have a test for that on `decimal_parsing` test of the `input_output` module. Because of this I decided to roll my own simple implementation of fixed point representation and have rust_decimal as an optional dependency: the program can run with rust_decimal or my own (module `fixed_point`). The default is `fixed_point` module implementation. Feel free to test with rust decimal by enabling `--features rust_decimal` on build. The limitations of my own implementation is that it supports 128 bits max and fixed precision (via const generics), rust_decimal supports 96 bits but flexible precision. I couldn't find a suitable fixed point implementation for rust that gives arbitrary precision, this seems an area where a good quality open source implementation is sorely needed.

## Safety and robustness
The dependencies where carefully selected to not introduce any unsafety or bloat. No unsafe code is used which is enforced by the `#![forbid(unsafe_code)]` lint check.
Error handling is extensive with detailed error generation (assisted by the excellent thiserror crate). Custom error types are used for the transaction processing "library". At the top level on the main crate and the integration tests the anyhow crate is used to facilitate the exposure of clear error messages to the user.
Malformation of the input csv file is not tolerated, as well as exceeding the precision of the money amounts. Any of these errors cause termination, as continuing would not be safe as the subsequent transactions will find the system in an inconsistent state.
Erroneous transactions are handled as per spec (ignoring withdrawals of more than available funds as well as ignoring disputing, resolving or charge-backs to nonexistent transactions or clients). A log message of info level is generated in these cases but the processing continues, as per spec. To enable log messages set the `RUST_LOG` environment variable as explained on the configuration section.

## Efficiency
Special care was taken to minimize the use of resources. The input csv file is processed line by line and never loaded in memory at the same time. Client accounts of course need to be kept to maintain their state, their are kept in memory in this implementation. A potential extension here could be to offload them to an external cache with overflow to a database in case the expected number of client accounts exceed available memory.

## Maintainability
The code was written using idiomatic Rust where possible, using descriptive names throughout. I feel that no sacrifice to maintainability had to be made to accommodate any other requirement, being performance or functionality.

## Modules description
- account: Keeps track of each client account ( `ClientAccount` ) and contains the logic to apply transactions. Each transaction pre-condition is checked. In case of errors during transaction apply a detailed error object is returned with the details.
- transaction: Models the different type or transaction as an enum with struct members ( `Transaction` ) to capture the data that each different transaction has.
- input_output: Handles serialization to/from CSV file. It defines a `TransactionRow` and `ClientRecord` which model the input and output csv formats respectively. Each of these is in turn converted to the `Transaction` and from `ClientAccount` objects. Modeling the format (via serde) and the actual business model objects were kept separate as a conscious design decision, to encapsulate the format and parsing logic in `TransactionRow`, `ClientRecord` while `Transaction` and `ClientAccount` model the business logic.
- fixed_point: Contains the custom fixed point representation. It's kept simple, as a newtype of a i128 or i64 or any other type that implements the `PrimInt` + `Signed` traits from the excellent num_traits crate. The precision is defined at compile time via a const generic argument. On the top level lib a `Money` type alias is defined for i128 and 4 digits of precision. 128 was deemed enough to represent monetary amounts (31 digits can be represented with 128 bits). An ideal implementation would allow arbitrary sized decimals (BigDecimal) but no satisfactory implementation was found on crates.io . Implementing full arbitrary size fixed point decimal was deemed overkill for this exercise.
- lib: Contains the top level utility functions to drive all the above modules: To process input transactions generating the client accounts and then writing the accounts state to the output file.
- main: Contains the command line interface handling. Argument parsing and using the lib utility functions.
