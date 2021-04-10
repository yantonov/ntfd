use crate::environment::Environment;
use crate::process::exec;
use std::path::PathBuf;

fn get_handler_executable(env: &Environment,
                          name: &str) -> Result<PathBuf, String> {
    let executable_dir = env.executable_dir();
    let handler_executable = executable_dir
        .join("conf")
        .join(name)
        .join("run");

    if handler_executable.exists() {
        Ok(handler_executable.clone())
    } else {
        let default_handler_executable = executable_dir
            .join("conf")
            .join("default")
            .join("run");
        if default_handler_executable.exists() {
            Ok(default_handler_executable.clone())
        } else {
            Err(format!("cannot find neither executable {} nor default executable {}",
                        handler_executable.to_str().unwrap(),
                        default_handler_executable.to_str().unwrap()))
        }
    }
}

pub fn execute(env: &Environment,
               name: String) -> Result<(), String> {
    let handler_executable = get_handler_executable(env, &name)?;
    exec(&env.shell(),
         &vec![
             "-c".to_string(),
             handler_executable.to_str().unwrap().to_string()
         ])
        .map(|_code| ())
}