use std::error::Error;
use std::process::Command;

pub fn run_target(target: &str) -> Result<(), Box<dyn Error>> {
    let mut command_parts = target.split_whitespace();
    let program = match command_parts.next() {
        Some(program) => program,
        None => return Ok(()),
    };

    let mut command = Command::new(program);
    command.args(command_parts);
    let _child = command.spawn()?;
    Ok(())
}
