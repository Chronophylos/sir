#![feature(or_patterns)]
#![feature(bool_to_option)]
#![feature(backtrace)]

use anyhow::{ensure, Context, Result};
use directories::ProjectDirs;
use log::info;
use self_update::{cargo_crate_version, Status::*};
use std::process::{exit, Command};

pub mod preferences;
pub mod workbook;

pub trait Column {
    fn try_into_index(&self) -> Result<u32>;
}

impl Column for &str {
    fn try_into_index(&self) -> Result<u32> {
        ensure!(!self.is_empty(), "Column is empty");

        let index = self
            .chars()
            .filter(char::is_ascii_alphabetic)
            .map(|c| c.to_digit(36).expect("char is expected to be alphabetic"))
            .fold(0u32, |acc, x| acc * 26 + x - 9);

        Ok(index - 1)
    }
}

impl Column for String {
    fn try_into_index(&self) -> Result<u32> {
        self.as_str().try_into_index()
    }
}

pub fn get_proj_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "chronophylos", "sir").context("No valid home directory found")
}

pub fn update(bin_name: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        info!("Running as dev: Skipping update check");
        return Ok(());
    } else {
        info!("Checking for update");
    }

    let status = self_update::backends::github::Update::configure()
        .repo_owner("Chronophylos")
        .repo_name("sir")
        .bin_name(bin_name)
        .show_output(false)
        .no_confirm(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    match status {
        UpToDate(_) => info!("{} is up to date", bin_name),
        Updated(version) => {
            info!("Updated {} to {}", bin_name, version);

            let mut args = std::env::args();

            let code = Command::new(args.next().unwrap()).spawn()?.wait()?;

            if code.success() == false {
                exit(code.code().unwrap_or(1));
            } else {
                exit(0);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column() {
        assert!("".try_into_index().is_err());
        assert_eq!("W".try_into_index().unwrap(), 22);
        assert_eq!("AA".try_into_index().unwrap(), 26);
        assert_eq!("CY".try_into_index().unwrap(), 102);
    }
}
