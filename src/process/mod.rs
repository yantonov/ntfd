use std::process::{Command};

fn pretty_printed_command(executable: &str,
                          args: &[String]) -> String
{
    let mut tokens: Vec<String> = vec![
        executable.to_string()
    ];
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

pub struct EnvVar {
    name: String,
    value: String,
}

impl EnvVar {
    pub fn new(name: &str, value: &str) -> EnvVar {
        EnvVar {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

pub fn exec(executable: &str,
            args: &[String],
            env_vars: &[EnvVar]) -> Result<ExecutionResult, String>
{
    let output = Command::new(executable)
        .args(args)
        .envs(env_vars.iter().map(|item| (item.name.clone(), item.value.clone())).into_iter())
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