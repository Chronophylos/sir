#![feature(never_type)]
#![feature(async_closure)]
#![feature(bool_to_option)]

use anyhow::{Error, Result};
use course_list::{CourseList, CourseListOptions};
use flexi_logger::{colored_detailed_format, detailed_format, Logger};
use iced::{
    button, executor, text_input, window, Align, Application, Button, Column, Command, Element,
    Length, Row, Settings, Space, Text, TextInput,
};
use log::{error, info};
use sir::{
    get_proj_dirs,
    preferences::{load_preferences, store_preferences, Preferences},
    update,
    workbook::WorkbookManager,
};

mod course_list;

fn main() -> Result<()> {
    let proj_dirs = get_proj_dirs()?;

    if cfg!(debug_assertions) {
        Logger::with_env_or_str("course_list_generator=debug, sir=debug, info")
            .duplicate_to_stderr(flexi_logger::Duplicate::Info)
            .format(colored_detailed_format)
    } else {
        Logger::with_str("info")
            .log_to_file()
            .directory(proj_dirs.data_dir().join("log"))
            .format(detailed_format)
    }
    .start()?;

    update("course_list_generator")?;

    info!("Starting window");
    Main::run(Settings {
        window: window::Settings {
            size: (900, 300),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    });

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum State {
    Entry,
    Error,
    Result,
}

impl Default for State {
    fn default() -> Self {
        Self::Entry
    }
}

#[derive(Debug, Clone)]
enum Message {
    Nothing,

    SrcPathInputChanged(String),
    SrcSheetInputChanged(String),
    SrcColumnInputChanged(String),

    DestPathInputChanged(String),

    GeneratePressed,
    BackPressed,

    AuxNameInputChanged { id: usize, value: String },
    AuxColInputChanged { id: usize, value: String },

    //GenerateCourseList,
    LoadPreferences(Preferences),
    StorePreferences(Result<(), String>),
}

#[derive(Default)]
struct Main {
    src_path_input: text_input::State,
    src_path_text: String,

    src_sheet_input: text_input::State,
    src_sheet_text: String,

    src_column_input: text_input::State,
    src_column_text: String,

    dest_path_input: text_input::State,
    dest_path_text: String,

    generate_button: button::State,
    back_button: button::State,

    aux_name_input: Vec<text_input::State>,
    aux_name_text: Vec<String>,
    aux_col_input: Vec<text_input::State>,
    aux_col_text: Vec<String>,

    error_text: String,
    result_text: String,

    state: State,

    workbook_manager: WorkbookManager,
}

const AUXILIARIES: usize = 2;

impl Application for Main {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                aux_name_text: vec![String::new(); AUXILIARIES],
                aux_name_input: vec![text_input::State::default(); AUXILIARIES],
                aux_col_text: vec![String::new(); AUXILIARIES],
                aux_col_input: vec![text_input::State::default(); AUXILIARIES],
                ..Self::default()
            },
            Command::perform(load_preferences(), |prefs| match prefs {
                Ok(prefs) => Message::LoadPreferences(prefs),
                Err(err) => {
                    error!("Could not load preferences: {}", err);
                    Message::Nothing
                }
            }),
        )
    }

    fn title(&self) -> String {
        format!(
            "SiR Course List Generator Version {} by {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS")
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use Message::*;

        match message {
            Nothing => {}

            SrcPathInputChanged(s) => self.src_path_text = s,
            SrcSheetInputChanged(s) => self.src_sheet_text = s,
            SrcColumnInputChanged(s) => self.src_column_text = s,
            DestPathInputChanged(s) => self.dest_path_text = s,

            GeneratePressed => {
                if let Err(err) = self.workbook_manager.open(&self.src_path_text) {
                    let err = Error::new(err);
                    self.error_text = format!("Could not open spreadsheet file: {:?}", err);
                    error!(
                        "Error opening workbook manager (path: {}): {:?}",
                        self.src_path_text, err
                    );
                    self.state = State::Error;
                    return Command::none();
                }

                let auxiliaries = self
                    .aux_name_text
                    .clone()
                    .into_iter()
                    .zip(self.aux_col_text.clone().into_iter())
                    .filter(|(name, col)| !(name.is_empty() && col.is_empty()))
                    .map(|(name, col)| (name, col))
                    .collect();
                let options = &CourseListOptions {
                    show_price: false,
                    auxiliaries,
                };

                let mut list = match self.workbook_manager.read_course_list(
                    &self.src_sheet_text,
                    &self.src_column_text,
                    options,
                ) {
                    Ok(l) => l,
                    Err(err) => {
                        self.error_text = format!("Could not read course list: {:#?}", err);
                        error!("Error reading course list: {:#?}", err);
                        self.state = State::Error;
                        return Command::none();
                    }
                };

                list.sort();

                self.result_text = format!(
                    "Successfully wrote data of {} participants to {}",
                    list.len(),
                    self.dest_path_text
                );

                if let Err(err) =
                    WorkbookManager::write_course_list(&self.dest_path_text, list, options)
                {
                    self.error_text = format!("Could not write course list: {:#?}", err);
                    error!(
                        "Error writing course list (path: {}): {:#?}",
                        self.dest_path_text, err
                    );
                    self.state = State::Error;
                    return Command::none();
                }

                self.state = State::Result;

                return Command::perform(
                    store_preferences(Preferences {
                        src_path: self.src_path_text.clone(),
                        src_sheet: self.src_sheet_text.clone(),
                        src_column: self.src_column_text.clone(),
                        dest_path: self.dest_path_text.clone(),
                        auxiliaries: Some(
                            self.aux_name_text
                                .clone()
                                .into_iter()
                                .zip(self.aux_col_text.clone().into_iter())
                                .collect(),
                        ),
                    }),
                    |result| Message::StorePreferences(result.map_err(|err| format!("{}", err))),
                );
            }
            BackPressed => match self.state {
                State::Error | State::Result => self.state = State::Entry,
                _ => {}
            },

            AuxNameInputChanged { id, value } => self.aux_name_text[id] = value,
            AuxColInputChanged { id, value } => self.aux_col_text[id] = value,

            LoadPreferences(prefs) => {
                self.src_path_text = prefs.src_path.to_string();
                self.src_sheet_text = prefs.src_sheet.to_string();
                self.src_column_text = prefs.src_column.to_string();
                self.dest_path_text = prefs.dest_path.to_string();

                if let Some(auxiliaries) = prefs.auxiliaries {
                    let (mut aux_name_text, mut aux_col_text): (Vec<String>, Vec<String>) =
                        auxiliaries.into_iter().unzip();

                    aux_name_text.resize(AUXILIARIES, String::new());
                    aux_col_text.resize(AUXILIARIES, String::new());

                    self.aux_name_text = aux_name_text;
                    self.aux_col_text = aux_col_text;
                }
            }
            StorePreferences(result) => {
                if let Err(err) = result {
                    error!("Could not store preferences: {}", err)
                }
            }
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        use State::*;

        let column = Column::new()
            .align_items(Align::Center)
            .padding(10)
            .spacing(10);

        match self.state {
            Entry => {
                let mut auxiliaries = (0..AUXILIARIES)
                    .zip(self.aux_name_input.iter_mut())
                    .zip(self.aux_name_text.iter())
                    .zip(self.aux_col_input.iter_mut())
                    .zip(self.aux_col_text.iter())
                    .map(|((((id, name_input), name_text), col_state), col_text)| {
                        Row::new()
                            .align_items(Align::Start)
                            .spacing(10)
                            .push(
                                TextInput::new(col_state, "XX", col_text, move |value| {
                                    Message::AuxColInputChanged { id, value }
                                })
                                .padding(5)
                                .width(Length::Units(30)),
                            )
                            .push(
                                TextInput::new(name_input, "Name", name_text, move |value| {
                                    Message::AuxNameInputChanged { id, value }
                                })
                                .padding(5)
                                .width(Length::Units(100)),
                            )
                            .into()
                    })
                    .collect::<Vec<Element<_>>>();

                auxiliaries.insert(0, Text::new("Additional Columns").into());
                auxiliaries.insert(1, Space::new(Length::Fill, Length::Units(10)).into());

                column
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .padding(20)
                            .spacing(10)
                            .push(Text::new("Source"))
                            .push(
                                TextInput::new(
                                    &mut self.src_path_input,
                                    "path to worksheet",
                                    &self.src_path_text,
                                    Message::SrcPathInputChanged,
                                )
                                .on_submit(Message::GeneratePressed)
                                .padding(5),
                            )
                            .push(Text::new("Sheet"))
                            .push(
                                TextInput::new(
                                    &mut self.src_sheet_input,
                                    "sheet name",
                                    &self.src_sheet_text,
                                    Message::SrcSheetInputChanged,
                                )
                                .on_submit(Message::GeneratePressed)
                                .padding(5)
                                .width(Length::Units(120)),
                            )
                            .push(Text::new("Column"))
                            .push(
                                TextInput::new(
                                    &mut self.src_column_input,
                                    "A",
                                    &self.src_column_text,
                                    Message::SrcColumnInputChanged,
                                )
                                .padding(5)
                                .width(Length::Units(30)),
                            ),
                    )
                    .push(
                        Row::with_children(auxiliaries)
                            .align_items(Align::Start)
                            .padding(20)
                            .spacing(10),
                    )
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .padding(20)
                            .spacing(10)
                            .push(Text::new("Destination"))
                            .push(
                                TextInput::new(
                                    &mut self.dest_path_input,
                                    "path to csv file",
                                    &self.dest_path_text,
                                    Message::DestPathInputChanged,
                                )
                                .padding(5),
                            ),
                    )
                    .push(Space::with_height(Length::Fill))
                    .push(
                        Button::new(&mut self.generate_button, Text::new("Generate"))
                            .on_press(Message::GeneratePressed),
                    )
            }
            Error => column
                .push(Row::new().push(Text::new(self.error_text.clone()).size(18)))
                .push(Space::with_height(Length::Fill))
                .push(
                    Row::new().align_items(Align::Start).push(
                        Button::new(&mut self.back_button, Text::new("Ok"))
                            .on_press(Message::BackPressed),
                    ),
                ),
            Result => column
                .push(Row::new().push(Text::new(self.result_text.clone()).size(20)))
                .push(Space::with_height(Length::Fill))
                .push(
                    Row::new().align_items(Align::Start).push(
                        Button::new(&mut self.back_button, Text::new("Back"))
                            .on_press(Message::BackPressed),
                    ),
                ),
        }
        .into()
    }
}
