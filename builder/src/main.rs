use std::process::ExitCode;

use containers_builder::command;

fn main() -> ExitCode {
    match command::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("エラー: {error}");
            ExitCode::FAILURE
        }
    }
}
