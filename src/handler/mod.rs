use crate::environment::Environment;
use crate::process::{exec, ExecutionResult, EnvVar};
use std::path::PathBuf;
use warp::hyper::body::Bytes;
use serde_json::{Value, Map};

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

fn env_vars(body_str: &str, json_body: &Value) -> Vec<EnvVar> {
    let mut result: Vec<EnvVar> = vec![];
    result.push(EnvVar::new("NTFD_JSON_BODY", body_str));
    match json_body {
        Value::Object(object) => {
            for key in object.keys().into_iter() {
                result.push(
                    EnvVar::new(format!("NTFD_JSON_FIELD_{}", key.to_uppercase()).as_str(),
                                object.get(key).unwrap().as_str().unwrap()))
            }
        }
        _ => {}
    }
    result
}

pub fn execute(env: &Environment,
               name: String,
               body: Bytes) -> Result<ExecutionResult, String> {
    let handler_executable = get_handler_executable(env, &name)?;
    let body_str = std::str::from_utf8(body.as_ref())
        .map_err(|_| "error converting bytes to &str")?;
    let json_body: Value = if body_str.len() == 0 {
        Value::Object(Map::new())
    } else {
        serde_json::from_str(body_str)
            .map_err(|_| "cannot parse json")?
    };
    let env_vars = env_vars(body_str, &json_body);
    exec(&env.shell(),
         &vec![
             "-c".to_string(),
             handler_executable.to_str().unwrap().to_string()
         ],
         &env_vars)
}