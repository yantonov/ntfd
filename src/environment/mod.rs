use std::env;
use std::path::{PathBuf};

pub struct Environment {
    executable_dir: PathBuf,
}

impl Environment {
    pub fn executable_dir(&self) -> &PathBuf {
        &self.executable_dir
    }

    #[cfg(test)]
    pub fn for_dir(dir: PathBuf) -> Environment {
        Environment {
            executable_dir: dir,
        }
    }
}

struct SystemEnvironment {}

impl SystemEnvironment {
    pub fn executable_dir(&self) -> Result<PathBuf, String> {
        let executable = env::current_exe()
            .map_err(|_| "cannot get current executable")?;
        match executable
            .parent()
            .map(|x| x.to_path_buf()) {
            None => Err("cannot get parent directory".to_string()),
            Some(v) => Ok(v),
        }
    }
}

pub fn system_environment() -> Result<Environment, String> {
    let sys_env = SystemEnvironment {};
    Ok(Environment {
        executable_dir: sys_env.executable_dir()?,
    })
}
