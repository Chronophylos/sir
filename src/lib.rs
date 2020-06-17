use anyhow::{ensure, Context, Result};
use calamine::DataType;
use std::{collections::HashMap, path::Path};

pub type Table = HashMap<String, Vec<Vec<DataType>>>;

pub fn read_course_list(path: &str, sheet_name: &str, column: &str) -> Result<Table> {
    use calamine::{open_workbook, Reader, Xlsx};

    ensure!(!path.is_empty(), "Path is not set");
    ensure!(!sheet_name.is_empty(), "Sheet name is not set");
    ensure!(!column.is_empty(), "Column is no set");

    let expanded = shellexpand::full(path)
        .context("Could not expand path")?
        .into_owned();

    let path = Path::new(&expanded)
        .canonicalize()
        .context("Could not canonicalize path")?;

    let mut excel: Xlsx<_> =
        open_workbook(path.to_str().context("Could not convert path to string")?)
            .context("Could not open Workbook")?;

    let range = excel
        .worksheet_range(sheet_name)
        .context("Could not find sheet")??;

    let column = column_to_usize(column)?;

    let map = range
        .rows()
        // skip header
        .skip(30)
        // filter out empty rows
        .filter(|data| data[column].is_string())
        // sort rows into hashmap
        .map(|data| (data[column].get_string().unwrap(), data))
        .fold(HashMap::new(), |mut acc, (course, data)| {
            let data: Vec<DataType> = data.into();
            acc.entry(course.to_owned()).or_insert(Vec::new());
            acc.entry(course.to_owned()).and_modify(|v| v.push(data));
            acc
        });

    Ok(map)
}

pub fn write_course_list(path: &str, table: Table) -> Result<()> {
    ensure!(!path.is_empty(), "Path is not set");

    let expanded = shellexpand::full(path)
        .context("Could not expand path")?
        .into_owned();

    let mut wtr = csv::Writer::from_path(expanded)?;

    wtr.write_record(&["Gruppe", "Kunden ID", "Name", "Telefon"])?;

    for key in table.keys() {
        for row in table.get(key).unwrap() {
            wtr.write_record(&[
                key,
                &row[0].to_string(),
                &row[2].to_string(),
                &row[7].to_string(),
            ])?;
        }
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
