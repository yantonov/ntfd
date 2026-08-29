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
                        handler_executable.display(),
                        default_handler_executable.display()))
        }
    }
}

fn is_valid_field_name(key: &str) -> bool {
    !key.is_empty()
        && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn env_vars(body_str: &str, json_body: &Value) -> Vec<EnvVar> {
    let mut result: Vec<EnvVar> = vec![
        EnvVar::new("NTFD_JSON_BODY", body_str)
    ];
    if let Value::Object(object) = json_body {
        for (key, value) in object {
            if !is_valid_field_name(key) {
                continue;
            }
            let value_str = match value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result.push(
                EnvVar::new(format!("NTFD_JSON_FIELD_{}", key.to_uppercase()).as_str(),
                            value_str.as_str()))
        }
    }
    result
}


pub fn configured_handlers(env: &Environment) -> Result<Vec<String>, String> {
    let conf_dir = env.executable_dir().join("conf");
    let entries = match std::fs::read_dir(&conf_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(vec![]),
    };
    let mut keys: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("run").exists())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    keys.sort();
    Ok(keys)
}

pub async fn execute(env: &Environment,
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
    exec(&handler_executable, &[], &env_vars).await
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
    fn numeric_field_is_passed_as_json_representation() {
        let body = r#"{"count":42}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        let field = vars.iter().find(|v| v.name() == "NTFD_JSON_FIELD_COUNT").unwrap();
        assert_eq!("42", field.value());
    }

    #[test]
    fn boolean_field_is_passed_as_json_representation() {
        let body = r#"{"enabled":true}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        let field = vars.iter().find(|v| v.name() == "NTFD_JSON_FIELD_ENABLED").unwrap();
        assert_eq!("true", field.value());
    }

    #[test]
    fn null_field_is_passed_as_json_representation() {
        let body = r#"{"payload":null}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        let field = vars.iter().find(|v| v.name() == "NTFD_JSON_FIELD_PAYLOAD").unwrap();
        assert_eq!("null", field.value());
    }

    #[test]
    fn nested_field_is_passed_as_json_representation() {
        let body = r#"{"outer":{"inner":1}}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        let field = vars.iter().find(|v| v.name() == "NTFD_JSON_FIELD_OUTER").unwrap();
        assert_eq!(r#"{"inner":1}"#, field.value());
    }

    #[test]
    fn mixed_field_types_do_not_panic() {
        let body = r#"{"a":"s","b":1,"c":true,"d":null,"e":[1,2]}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(6, vars.len());
    }

    #[test]
    fn field_name_with_an_equals_sign_is_skipped() {
        let body = r#"{"a=b":"x","ok":"y"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(2, vars.len());
        assert!(vars.iter().any(|v| v.name() == "NTFD_JSON_FIELD_OK"));
    }

    #[test]
    fn field_name_with_a_nul_byte_is_skipped() {
        let body = r#"{"a\u0000b":"x"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(1, vars.len());
    }

    #[test]
    fn field_name_with_a_dash_is_skipped() {
        let body = r#"{"a-b":"x"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(1, vars.len());
    }

    #[test]
    fn field_name_with_a_space_is_skipped() {
        let body = r#"{"a b":"x"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(1, vars.len());
    }

    #[test]
    fn empty_field_name_is_skipped() {
        let body = r#"{"":"x"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(1, vars.len());
    }

    #[test]
    fn skipped_field_still_reaches_the_handler_through_the_raw_body() {
        let body = r#"{"a-b":"x"}"#;
        let json: Value = serde_json::from_str(body).unwrap();
        let vars = env_vars(body, &json);
        assert_eq!(body, vars[0].value());
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

    #[test]
    fn specific_handler_is_found_when_it_exists() {
        let dir = tempfile::tempdir().unwrap();
        let handler_dir = dir.path().join("conf").join("mykey");
        std::fs::create_dir_all(&handler_dir).unwrap();
        std::fs::File::create(handler_dir.join("run")).unwrap();

        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        let result = get_handler_executable(&env, "mykey").unwrap();
        assert_eq!(handler_dir.join("run"), result);
    }

    #[test]
    fn default_handler_is_used_when_specific_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let default_dir = dir.path().join("conf").join("default");
        std::fs::create_dir_all(&default_dir).unwrap();
        std::fs::File::create(default_dir.join("run")).unwrap();

        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        let result = get_handler_executable(&env, "nonexistent").unwrap();
        assert_eq!(default_dir.join("run"), result);
    }

    #[test]
    fn specific_handler_takes_priority_over_default() {
        let dir = tempfile::tempdir().unwrap();
        let specific_dir = dir.path().join("conf").join("mykey");
        let default_dir = dir.path().join("conf").join("default");
        std::fs::create_dir_all(&specific_dir).unwrap();
        std::fs::create_dir_all(&default_dir).unwrap();
        std::fs::File::create(specific_dir.join("run")).unwrap();
        std::fs::File::create(default_dir.join("run")).unwrap();

        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        let result = get_handler_executable(&env, "mykey").unwrap();
        assert_eq!(specific_dir.join("run"), result);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handler_runs_when_the_install_path_contains_a_space() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("my tools");
        let handler_dir = dir.join("conf").join("spaced");
        std::fs::create_dir_all(&handler_dir).unwrap();
        let run = handler_dir.join("run");
        let mut file = std::fs::File::create(&run).unwrap();
        file.write_all(b"#!/bin/sh
echo started
").unwrap();
        drop(file);
        std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o755)).unwrap();

        let env = crate::environment::Environment::for_dir(dir);
        let result = execute(&env, "spaced".to_string(), Bytes::new()).await.unwrap();
        assert_eq!(0, result.code());
        assert_eq!("started
", result.stdout());
    }

    #[test]
    fn configured_handlers_lists_directories_holding_a_run_file() {
        let dir = tempfile::tempdir().unwrap();
        for key in ["zulu", "alpha"] {
            let handler_dir = dir.path().join("conf").join(key);
            std::fs::create_dir_all(&handler_dir).unwrap();
            std::fs::File::create(handler_dir.join("run")).unwrap();
        }
        std::fs::create_dir_all(dir.path().join("conf").join("empty")).unwrap();

        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        assert_eq!(vec!["alpha".to_string(), "zulu".to_string()],
                   configured_handlers(&env).unwrap());
    }

    #[test]
    fn configured_handlers_is_empty_without_a_conf_directory() {
        let dir = tempfile::tempdir().unwrap();
        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        assert!(configured_handlers(&env).unwrap().is_empty());
    }

    #[test]
    fn error_is_returned_when_no_handler_exists() {
        let dir = tempfile::tempdir().unwrap();
        let env = crate::environment::Environment::for_dir(dir.path().to_path_buf());
        assert!(get_handler_executable(&env, "missing").is_err());
    }
}
