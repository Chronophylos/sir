#![feature(or_patterns)]
#![feature(bool_to_option)]
#![feature(backtrace)]

use anyhow::{ensure, Context, Result};
use directories::ProjectDirs;

pub mod preferences;
pub mod workbook;

pub fn column_to_usize(column: &str) -> Result<usize> {
    let column = column.to_owned();

    ensure!(!column.is_empty(), "Column is empty");

    Ok(column
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        // convert to a number but ignore actual numbers
        .map(|c| c.to_digit(36).unwrap() - 9)
        .fold(0, |acc, x| {
            acc * 26 + x as usize
        })
        // use zero indexing
        - 1)
}

pub fn get_proj_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("com", "chronophylos", "sir").context("No valid home directory found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_to_usize() {
        assert_eq!(column_to_usize("W").unwrap(), 22);
        assert_eq!(column_to_usize("AA").unwrap(), 26);
        assert_eq!(column_to_usize("CY").unwrap(), 102);
    }
}
