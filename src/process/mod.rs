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
            args: &Vec<String>) -> Result<ExecutionResult, String>
{
    let output = Command::new(executable)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             pretty_printed_command(executable, args),
                             e))?;
    Ok(ExecutionResult {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    })
}