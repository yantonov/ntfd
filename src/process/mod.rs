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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_result_returns_code() {
        let result = ExecutionResult { code: 42, stdout: String::new(), stderr: String::new() };
        assert_eq!(42, result.code());
    }

    #[test]
    fn execution_result_returns_stdout() {
        let result = ExecutionResult { code: 0, stdout: "hello".to_string(), stderr: String::new() };
        assert_eq!("hello", result.stdout());
    }

    #[test]
    fn execution_result_returns_stderr() {
        let result = ExecutionResult { code: 0, stdout: String::new(), stderr: "oops".to_string() };
        assert_eq!("oops", result.stderr());
    }

    #[test]
    fn env_var_stores_name() {
        let var = EnvVar::new("MY_VAR", "value");
        assert_eq!("MY_VAR", var.name());
    }

    #[test]
    fn env_var_stores_value() {
        let var = EnvVar::new("MY_VAR", "my_value");
        assert_eq!("my_value", var.value());
    }

    #[test]
    fn exec_returns_err_for_missing_executable() {
        let result = exec("/nonexistent/binary", &[], &[]);
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn exec_captures_zero_exit_code() {
        let result = exec("/bin/sh", &["-c".to_string(), "exit 0".to_string()], &[]).unwrap();
        assert_eq!(0, result.code());
    }

    #[cfg(unix)]
    #[test]
    fn exec_captures_nonzero_exit_code() {
        let result = exec("/bin/sh", &["-c".to_string(), "exit 42".to_string()], &[]).unwrap();
        assert_eq!(42, result.code());
    }

    #[cfg(unix)]
    #[test]
    fn exec_captures_stdout() {
        let result = exec("/bin/sh", &["-c".to_string(), "echo hello".to_string()], &[]).unwrap();
        assert_eq!("hello\n", result.stdout());
    }

    #[cfg(unix)]
    #[test]
    fn exec_captures_stderr() {
        let result = exec("/bin/sh", &["-c".to_string(), "echo error >&2".to_string()], &[]).unwrap();
        assert_eq!("error\n", result.stderr());
    }

    #[cfg(unix)]
    #[test]
    fn exec_passes_env_vars_to_subprocess() {
        let vars = vec![EnvVar::new("TEST_VAR", "hello_world")];
        let result = exec("/bin/sh", &["-c".to_string(), "echo $TEST_VAR".to_string()], &vars).unwrap();
        assert_eq!("hello_world\n", result.stdout());
    }
}

pub fn exec(executable: &str,
            args: &[String],
            env_vars: &[EnvVar]) -> Result<ExecutionResult, String>
{
    let output = Command::new(executable)
        .args(args)
        .envs(env_vars.iter().map(|item| (item.name(), item.value())))
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