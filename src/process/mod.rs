use std::process::{Command};

fn pretty_printed_command(executable: &str,
                          args: &Vec<String>) -> String
{
    let mut tokens: Vec<String> = vec![];
    tokens.push(executable.to_string());
    for arg in args {
        tokens.push(arg.clone());
    }
    tokens.join(" ")
}

pub struct ExecutionResult {
    code: i32,
    stdout: String,
    stderr: String,
}

impl ExecutionResult {
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}

pub fn exec(executable: &str,
            args: &Vec<String>) -> ExecutionResult
{
    let output = Command::new(executable)
        .args(args)
        .output();

    match output {
        Ok(value) => {
            ExecutionResult {
                code: value.status.code().unwrap(),
                stdout: String::from_utf8(value.stdout).unwrap(),
                stderr: String::from_utf8(value.stderr).unwrap(),
            }
        }
        Err(e) => {
            ExecutionResult {
                code: -1,
                stdout: "".to_string(),
                stderr: format!("Failed to execute process [{}]. {}",
                                pretty_printed_command(executable, args),
                                e),
            }
        }
    }
}