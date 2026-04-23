use bytes::Bytes;
use crate::environment::Environment;
use crate::process::{exec, ExecutionResult, EnvVar};
use std::path::PathBuf;
use serde_json::{Value, Map};

fn get_handler_executable(env: &Environment,
                          name: &str) -> Result<PathBuf, String> {
    let executable_dir = env.executable_dir();
    let handler_executable = executable_dir
        .join("conf")
        .join(name)
        .join("run");

    if handler_executable.exists() {
        Ok(handler_executable)
    } else {
        let default_handler_executable = executable_dir
            .join("conf")
            .join("default")
            .join("run");
        if default_handler_executable.exists() {
            Ok(default_handler_executable)
        } else {
            Err(format!("cannot find neither executable {} nor default executable {}",
                        handler_executable.to_str().unwrap(),
                        default_handler_executable.to_str().unwrap()))
        }
    }
}

fn env_vars(body_str: &str, json_body: &Value) -> Vec<EnvVar> {
    let mut result: Vec<EnvVar> = vec![
        EnvVar::new("NTFD_JSON_BODY", body_str)
    ];
    if let Value::Object(object) = json_body {
        for key in object.keys().into_iter() {
            result.push(
                EnvVar::new(format!("NTFD_JSON_FIELD_{}", key.to_uppercase()).as_str(),
                object.get(key).unwrap().as_str().unwrap()))
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_body_yields_only_json_body_var() {
        let vars = env_vars("", &Value::Object(Map::new()));
        assert_eq!(1, vars.len());
        assert_eq!("NTFD_JSON_BODY", vars[0].name());
        assert_eq!("", vars[0].value());
    }

    #[test]
    fn json_string_field_is_extracted_as_env_var() {
        let body = r#"{"title":"hello"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(2, vars.len());
        let field = vars.iter().find(|v| v.name() == "NTFD_JSON_FIELD_TITLE").unwrap();
        assert_eq!("hello", field.value());
    }

    #[test]
    fn json_field_names_are_uppercased() {
        let body = r#"{"lowercase":"value"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert!(vars.iter().any(|v| v.name() == "NTFD_JSON_FIELD_LOWERCASE"));
    }

    #[test]
    fn multiple_json_fields_are_all_extracted() {
        let body = r#"{"a":"1","b":"2"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(3, vars.len());
    }

    #[test]
    fn non_object_json_yields_only_body_var() {
        let body = r#""just a string""#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(1, vars.len());
        assert_eq!("NTFD_JSON_BODY", vars[0].name());
    }

    #[test]
    fn raw_json_body_is_preserved_in_body_var() {
        let body = r#"{"key":"val"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(body, vars[0].value());
    }
}

pub fn execute(env: &Environment,
               name: String,
               body: Bytes) -> Result<ExecutionResult, String> {
    let handler_executable = get_handler_executable(env, &name)?;
    let body_str = std::str::from_utf8(body.as_ref())
        .map_err(|_| "error converting bytes to &str")?;
    let json_body: Value = if body_str.is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str(body_str)
            .map_err(|_| "cannot parse json")?
    };
    let env_vars = env_vars(body_str, &json_body);
    exec(&env.shell(),
         &[
             "-c".to_string(),
             handler_executable.to_str().unwrap().to_string()
         ],
         &env_vars)
}