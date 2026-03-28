use super::Command;
use std::error::Error;

/// Test command implementation
pub struct TestCommand {
    pub message: String,
}

impl Command for TestCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Test command executed: {}", self.message);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "Test command for verifying the command pattern"
    }
}
