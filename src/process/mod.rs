use std::process::Command;

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

pub fn exec(executable: &str,
            args: &Vec<String>) -> Result<Option<i32>, String>
{
    let pretty_printed_command = pretty_printed_command(executable, args);

    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             pretty_printed_command,
                             e))?;

    output.wait()
        .map(|r| r.code())
        .map_err(|e| format!("Failed to wait child process [{}]. {}",
                             pretty_printed_command,
                             e))
}