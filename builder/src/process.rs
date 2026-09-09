use std::process::{Command, Output};

use crate::error::{Error, Result};

pub fn status(command: &mut Command, name: &'static str) -> Result<()> {
    let status = command.status().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed { command: name })
    }
}

pub fn output(command: &mut Command, name: &'static str) -> Result<Output> {
    let output = command.output().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(Error::CommandFailed { command: name })
    }
}
