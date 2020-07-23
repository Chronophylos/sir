use calamine::{open_workbook_auto, DataType, Range, Reader, Sheets};

use std::{fmt::Debug, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkbookError {
    #[error("Could not read workbook: {0}")]
    ReadWorkbook(#[source] calamine::Error),

    #[error("Could not get worksheet: {0}")]
    GetSheet(#[source] calamine::Error),

    #[error("No sheet found")]
    NoSheet,
}

#[derive(Default)]
pub struct WorkbookManager {
    reader: Option<Sheets>,
}

impl WorkbookManager {
    pub fn new() -> Self {
        Self { reader: None }
    }

    pub fn open<P>(&mut self, path: P) -> Result<(), WorkbookError>
    where
        P: AsRef<Path>,
    {
        self.reader =
            Some(open_workbook_auto(path).map_err(|err| WorkbookError::ReadWorkbook(err))?);
        Ok(())
    }

    pub fn sheets(&self) -> Option<&[String]> {
        self.reader.as_ref().map(|r| r.sheet_names())
    }

    pub fn get_sheet(&mut self, sheet: &str) -> Option<Result<Range<DataType>, WorkbookError>> {
        self.reader.as_mut().map(|r| {
            r.worksheet_range(sheet)
                .ok_or(WorkbookError::NoSheet)?
                .map_err(|err| WorkbookError::GetSheet(err))
        })
    }
}
