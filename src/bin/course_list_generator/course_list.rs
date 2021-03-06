use calamine::{DataType, Range, Reader, Sheets};
use sir::{
    workbook::{WorkbookError, WorkbookManager},
    Column,
};
use std::{cmp::Ordering, error::Error, num::TryFromIntError};
use thiserror::Error;
use xlsxwriter::{Format, FormatAlignment, FormatBorder, Workbook, Worksheet, XlsxError};

#[derive(Debug, Error)]
pub enum CourseListError {
    #[error("No Worksheet loaded")]
    NoReader,

    #[error("Could not convert column name to number: {0}")]
    ConvertColumn(#[source] anyhow::Error),

    #[error("Workbook Error: {0}")]
    WorkbookError(
        #[from]
        #[source]
        WorkbookError,
    ),

    #[error("Workbook Error: {0}")]
    ConversionError(
        #[from]
        #[source]
        TryFromIntError,
    ),

    #[error("Could not add a new worksheet to workbook: {0}")]
    AddWorksheet(#[source] XlsxError),

    #[error("Could not write header row: {0}")]
    WriteHeaderRow(#[source] XlsxError),

    #[error("Could not write data row: {0}")]
    WriteEntryRow(#[source] XlsxError),

    #[error("Could not deserialize course list: {0}")]
    Deserialize(#[source] Box<dyn Error>),

    #[error("Could not set column format: {0}")]
    SetColumn(#[source] XlsxError),
}

#[derive(Debug)]
pub struct CourseEntry {
    id: i32,
    group: String,
    name: String,
    telephone: String,
    email: String,
    auxiliaries: Vec<String>,
}

impl Eq for CourseEntry {}

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

#[derive(Debug, Default)]
pub struct CourseListOptions {
    pub show_price: bool,
    pub auxiliaries: Vec<(String, String)>,
}

pub trait CourseList<R>
where
    R: Reader,
{
    fn sheets(&self) -> Option<&[String]>;
    fn get_sheet(&mut self, name: &str) -> Option<Result<Range<DataType>, WorkbookError>>;

    fn read_course_list(
        &mut self,
        sheet: &str,
        column: &str,
        options: &CourseListOptions,
    ) -> Result<Vec<CourseEntry>, CourseListError> {
        let column: usize = column
            .try_into_index()
            .map_err(|err| CourseListError::ConvertColumn(err))?;

        let range = self.get_sheet(sheet).ok_or(CourseListError::NoReader)??;

        let list = range
            .rows()
            // skip header
            .skip(30)
            // filter out rows without an entry a this column
            .filter(|data| data[column].is_string())
            // sort rows into hashmap
            .map(|data| {
                Ok(CourseEntry {
                    id: data[0].to_string().parse()?,
                    group: data[column].to_string(),
                    name: data[2].to_string(),
                    telephone: data[7].to_string().replace("\r\n", ";"),
                    email: data[11].to_string(),
                    auxiliaries: options
                        .auxiliaries
                        .iter()
                        .filter_map(|(_, column)| {
                            column
                                .try_into_index()
                                .ok()
                                .map(|column| data[column].to_string())
                        })
                        .collect(),
                })
            })
            .collect::<Result<Vec<CourseEntry>, Box<dyn Error>>>()
            .map_err(|err| CourseListError::Deserialize(err))?;

        Ok(list)
    }

    fn write_course_list(
        path: &str,
        list: Vec<CourseEntry>,
        options: &CourseListOptions,
    ) -> Result<(), CourseListError> {
        let workbook = Workbook::new(path);

        let header_format = workbook
            .add_format()
            .set_align(FormatAlignment::Center)
            .set_border_bottom(FormatBorder::Medium)
            .set_bold();

        /*
        let currency_format = workbook
            .add_format()
            .set_num_format("#,##0.00 €;-#,##0.00 €");
        */

        let id_format = workbook.add_format().set_align(FormatAlignment::Center);

        let mut sheet = workbook
            .add_worksheet(None)
            .map_err(|err| CourseListError::AddWorksheet(err))?;

        sheet
            .set_column(0, 0, 5., Some(&id_format))
            .map_err(|err| CourseListError::SetColumn(err))?;
        sheet
            .set_column(1, 1, 30., None)
            .map_err(|err| CourseListError::SetColumn(err))?;
        sheet
            .set_column(2, 2, 20., None)
            .map_err(|err| CourseListError::SetColumn(err))?;
        sheet
            .set_column(3, 3, 15., None)
            .map_err(|err| CourseListError::SetColumn(err))?;
        sheet
            .set_column(4, 4, 30., None)
            .map_err(|err| CourseListError::SetColumn(err))?;
        sheet
            .set_column(5, 6, 30., None)
            .map_err(|err| CourseListError::SetColumn(err))?;

        sheet.write_header(0, 0, &options, Some(&header_format))?;
        sheet.write_rows(1, 0, list, &options)?;

        Ok(())
    }
}

impl CourseList<Sheets> for WorkbookManager {
    fn sheets(&self) -> Option<&[String]> {
        self.sheets()
    }

    fn get_sheet(&mut self, name: &str) -> Option<Result<Range<DataType>, WorkbookError>> {
        self.get_sheet(name)
    }
}

trait CourseListWriter {
    fn write_header(
        &mut self,
        row: u32,
        col: u16,
        options: &CourseListOptions,
        format: Option<&Format>,
    ) -> Result<(), CourseListError>;

    fn write_rows(
        &mut self,
        row: u32,
        col: u16,
        entries: Vec<CourseEntry>,
        options: &CourseListOptions,
    ) -> Result<(), CourseListError>;
}

const HEADERS: [&'static str; 5] = ["Kundennummer", "Gruppe", "Name", "Telefon", "E-Mail"];

impl CourseListWriter for Worksheet<'_> {
    fn write_header(
        &mut self,
        row: u32,
        col: u16,
        options: &CourseListOptions,
        format: Option<&Format>,
    ) -> Result<(), CourseListError> {
        HEADERS
            .iter()
            .map(|&h| h)
            .chain(options.auxiliaries.iter().map(|(name, _)| name.as_str()))
            .enumerate()
            .map(|(i, header)| {
                self.write_string(row, col + i as u16, header, format)
                    .map_err(|err| CourseListError::WriteHeaderRow(err))
            })
            .collect()
    }

    fn write_rows(
        &mut self,
        row: u32,
        col: u16,
        entries: Vec<CourseEntry>,
        _options: &CourseListOptions,
    ) -> Result<(), CourseListError> {
        entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let row = row + i as u32;
                let mut col = col;

                self.write_number(row, col, entry.id.into(), None)
                    .map_err(|err| CourseListError::WriteEntryRow(err))?;
                col += 1;

                self.write_string(row, col, &entry.group, None)
                    .map_err(|err| CourseListError::WriteEntryRow(err))?;
                col += 1;

                self.write_string(row, col, &entry.name, None)
                    .map_err(|err| CourseListError::WriteEntryRow(err))?;
                col += 1;

                self.write_string(row, col, &entry.telephone, None)
                    .map_err(|err| CourseListError::WriteEntryRow(err))?;
                col += 1;

                self.write_string(row, col, &entry.email, None)
                    .map_err(|err| CourseListError::WriteEntryRow(err))?;
                col += 1;

                for text in &entry.auxiliaries {
                    self.write_string(row, col, text, None)
                        .map_err(|err| CourseListError::WriteEntryRow(err))?;
                    col += 1;
                }

                Ok(())
            })
            .collect()
    }
}
