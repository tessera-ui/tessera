use std::{collections::HashMap, ffi::OsString, fmt::Debug, path::Path};

use thiserror::Error;

use crate::util::cli::{Report, Reportable};

pub trait ExplicitEnv: Debug {
    fn explicit_env(&self) -> HashMap<String, OsString>;

    fn removed_env_vars(&self) -> Vec<String> {
        Vec::new()
    }

    fn configure_command(&self, command: duct::Expression) -> duct::Expression {
        let values = self.explicit_env();
        let removed = self.removed_env_vars();
        command.before_spawn(move |command| {
            command.envs(&values);
            for key in &removed {
                command.env_remove(key);
            }
            Ok(())
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The `{0}` environment variable isn't set, which is quite weird")]
    NotSet(&'static str),
}

impl Reportable for Error {
    fn report(&self) -> Report {
        Report::error("Failed to initialize base environment", self)
    }
}

#[derive(Clone, Debug)]
pub struct Env {
    vars: HashMap<String, std::ffi::OsString>,
}

impl Env {
    pub fn new() -> Result<Self, Error> {
        let mut vars = HashMap::new();

        let home = std::env::var_os("HOME").ok_or(Error::NotSet("HOME"))?;
        let path = std::env::var_os("PATH").ok_or(Error::NotSet("PATH"))?;
        if let Some(term) = std::env::var_os("TERM") {
            vars.insert("TERM".into(), term);
        }
        if let Some(ssh_auth_sock) = std::env::var_os("SSH_AUTH_SOCK") {
            vars.insert("SSH_AUTH_SOCK".into(), ssh_auth_sock);
        }

        vars.insert("HOME".into(), home);
        vars.insert("PATH".into(), path);

        Ok(Self { vars })
    }

    pub fn path(&self) -> &OsString {
        self.vars.get("PATH").unwrap()
    }

    pub fn prepend_to_path(mut self, path: impl AsRef<Path>) -> Self {
        let mut path = path.as_ref().as_os_str().to_os_string();
        path.push(":");
        path.push(self.path().clone());
        self.vars.insert("PATH".into(), path);
        self
    }

    pub fn insert_env_var(&mut self, key: String, value: OsString) {
        self.vars.insert(key, value);
    }

    pub fn explicit_env_vars(mut self, vars: HashMap<String, OsString>) -> Self {
        self.vars.extend(vars);
        self
    }
}

impl ExplicitEnv for Env {
    fn explicit_env(&self) -> HashMap<String, OsString> {
        self.vars.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct CommandEnv;

    impl ExplicitEnv for CommandEnv {
        fn explicit_env(&self) -> HashMap<String, OsString> {
            HashMap::from([("TESSERA_COMMAND_VALUE".to_owned(), "configured".into())])
        }

        fn removed_env_vars(&self) -> Vec<String> {
            vec!["TESSERA_COMMAND_REMOVED".to_owned()]
        }
    }

    #[test]
    fn command_environment_child() {
        if std::env::var_os("TESSERA_COMMAND_CHILD").is_some() {
            println!(
                "command-value={}",
                std::env::var("TESSERA_COMMAND_VALUE").unwrap()
            );
            println!(
                "command-removed={}",
                std::env::var_os("TESSERA_COMMAND_REMOVED").is_none()
            );
        }
    }

    #[test]
    fn command_environment_overrides_and_removals_are_applied_only_to_the_child() {
        let before = std::env::var_os("TESSERA_COMMAND_VALUE");
        let command = duct::cmd(
            std::env::current_exe().unwrap(),
            [
                "--exact",
                "env::tests::command_environment_child",
                "--nocapture",
            ],
        )
        .env("TESSERA_COMMAND_CHILD", "1")
        .env("TESSERA_COMMAND_VALUE", "stale")
        .env("TESSERA_COMMAND_REMOVED", "stale");
        let output = CommandEnv
            .configure_command(command)
            .stdout_capture()
            .run()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("command-value=configured"), "{stdout}");
        assert!(stdout.contains("command-removed=true"), "{stdout}");
        assert_eq!(std::env::var_os("TESSERA_COMMAND_VALUE"), before);
    }
}
