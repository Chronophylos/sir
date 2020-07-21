#![feature(or_patterns)]
#![feature(bool_to_option)]

use anyhow::{bail, ensure, Context, Result};
use calamine::{DataType, Range};
use directories::ProjectDirs;
use log::info;
use serde::Serialize;
use std::{cmp::Ordering, path::Path};

pub mod preferences;

#[derive(Debug, Serialize, Eq)]
pub struct CourseEntry {
    #[serde(rename = "Kundennummer")]
    id: String,
    #[serde(rename = "Gruppe")]
    group: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Telefon")]
    telephone: String,
    #[serde(rename = "E-Mail")]
    email: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "Restbetrag")]
    price: Option<String>,
}

impl Ord for CourseEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name).then(self.id.cmp(&other.id))
    }
}

impl PartialOrd for CourseEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CourseEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name) && self.id.eq(&other.id)
    }
}

pub fn read_course_list(
    path: &str,
    show_price: bool,
    sheet_name: &str,
    column: &str,
) -> Result<Vec<CourseEntry>> {
    ensure!(!path.is_empty(), "Path is not set");
    ensure!(!sheet_name.is_empty(), "Sheet name is not set");
    ensure!(!column.is_empty(), "Column is no set");

    let expanded = shellexpand::full(path)
        .context("Could not expand path")?
        .into_owned();

    let path = Path::new(&expanded)
        .canonicalize()
        .context("Could not canonicalize path")?;

    let range = match path.extension().map(|oss| oss.to_str()).flatten() {
        Some("xlsx" | "xlsm" | "xlam") => get_worksheet_range_xlsx(path, sheet_name),
        Some("ods") => get_worksheet_range_ods(path, sheet_name),
        None => bail!("File has no extension"),
        _ => bail!("Unsupported file extension"),
    }
    .context("Could not read worksheet range")?;

    let column = column_to_usize(column)?;

    let list = range
        .rows()
        // skip header
        .skip(30)
        // filter out empty rows
        .filter(|data| data[column].is_string())
        // sort rows into hashmap
        .map(|data| CourseEntry {
            id: data[0].to_string(),
            group: data[column].to_string(),
            name: data[2].to_string(),
            telephone: data[7].to_string(),
            email: data[11].to_string(),
            price: show_price.then(|| data[column + 8].to_string()),
        })
        .collect();

    Ok(list)
}

fn get_worksheet_range_xlsx<P>(path: P, sheet_name: &str) -> Result<Range<DataType>>
where
    P: AsRef<Path>,
{
    use calamine::{open_workbook, Reader, Xlsx};

    let path = path.as_ref();

    info!("Opening Workbook as Xlsx (path: {})", path.display());

    let mut workbook: Xlsx<_> =
        open_workbook(path.to_str().context("Could not convert path to string")?)
            .context("Could not open Workbook")?;

    Ok(workbook
        .worksheet_range(sheet_name)
        .context("Could not find sheet")??)
}

fn get_worksheet_range_ods<P>(path: P, sheet_name: &str) -> Result<Range<DataType>>
where
    P: AsRef<Path>,
{
    use calamine::{open_workbook, Ods, Reader};

    let path = path.as_ref();

    info!("Opening Workbook as Ods (path: {})", path.display());

    let mut workbook: Ods<_> =
        open_workbook(path.to_str().context("Could not convert path to string")?)
            .context("Could not open Workbook")?;

    Ok(workbook
        .worksheet_range(sheet_name)
        .context("Could not find sheet")??)
}

pub fn write_course_list(path: &str, table: Vec<CourseEntry>) -> Result<()> {
    use csv::WriterBuilder;

    ensure!(!path.is_empty(), "Path is not set");

    let expanded = shellexpand::full(path)
        .context("Could not expand path")?
        .into_owned();

    info!("Wrinting course list to {}", expanded);

    let mut wtr = WriterBuilder::new().has_headers(true).from_path(expanded)?;

    for entry in table {
        wtr.serialize(entry)
            .context("Could not serialize CouseEntry")?;
    }

    wtr.flush()?;

    Ok(())
}

fn column_to_usize(column: &str) -> Result<usize> {
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
