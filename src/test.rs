use std::env;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;

use crate::common::XWinOptions;
use crate::Build;

/// Execute all unit and integration tests and build examples of a local package
#[derive(Clone, Debug, Default, Parser)]
#[clap(
    setting = clap::AppSettings::DeriveDisplayOrder,
    trailing_var_arg = true,
    after_help = "Run `cargo help test` for more detailed information.\nRun `cargo test -- --help` for test binary options.")
]
pub struct Test {
    #[clap(flatten)]
    pub xwin: XWinOptions,

    #[clap(flatten)]
    pub cargo: cargo_options::Test,
}

impl Test {
    /// Create a new test from manifest path
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(manifest_path: Option<PathBuf>) -> Self {
        let mut build = Self::default();
        build.manifest_path = manifest_path;
        build
    }

    /// Execute `cargo test` command
    pub fn execute(&self) -> Result<()> {
        let build = Build {
            cargo: self.cargo.clone().into(),
            ..Default::default()
        };
        let mut test = build.build_command("test")?;
        if self.cargo.doc {
            test.arg("--doc");
        }
        if self.cargo.no_run {
            test.arg("--no-run");
        }
        if self.cargo.no_fail_fast {
            test.arg("--no-fail-fast");
        }
        if let Some(test_name) = self.cargo.test_name.as_ref() {
            test.arg(test_name);
        }
        if !self.cargo.args.is_empty() {
            test.arg("--");
            test.args(&self.cargo.args);
        }

        for target in &self.cargo.target {
            if target.contains("msvc") {
                if env::var_os("WINEDEBUG").is_none() {
                    test.env("WINEDEBUG", "-all");
                }
                let env_target = target.to_uppercase().replace('-', "_");
                let runner_env = format!("CARGO_TARGET_{}_RUNNER", env_target);
                if env::var_os(&runner_env).is_none() {
                    test.env(runner_env, "wine");
                }
            }
        }

        let mut child = test.spawn().context("Failed to run cargo test")?;
        let status = child.wait().expect("Failed to wait on cargo test process");
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
        Ok(())
    }
}

impl Deref for Test {
    type Target = cargo_options::Test;

    fn deref(&self) -> &Self::Target {
        &self.cargo
    }
}

impl DerefMut for Test {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cargo
    }
}

impl From<cargo_options::Test> for Test {
    fn from(cargo: cargo_options::Test) -> Self {
        Self {
            cargo,
            ..Default::default()
        }
    }
}
