#![feature(never_type)]
#![feature(async_closure)]
#![feature(bool_to_option)]

use anyhow::{Error, Result};
use course_list::{CourseList, CourseListOptions};
use flexi_logger::{detailed_format, Logger};
use iced::{
    button, executor, text_input, window, Align, Application, Button, Checkbox, Column, Command,
    Element, Length, Row, Settings, Space, Text, TextInput,
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
    Logger::with_env_or_str("course_list_generator=debug, sir=debug, info")
        .log_to_file()
        .directory(proj_dirs.data_dir().join("log"))
        .format(detailed_format)
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

    ShowPriceToggled(bool),

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

    show_price: bool,

    error_text: String,
    result_text: String,

    state: State,

    workbook_manager: WorkbookManager,
}

impl Application for Main {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self::default(),
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
        match message {
            Message::Nothing => {}
            Message::SrcPathInputChanged(s) => self.src_path_text = s,
            Message::SrcSheetInputChanged(s) => self.src_sheet_text = s,
            Message::SrcColumnInputChanged(s) => self.src_column_text = s,
            Message::DestPathInputChanged(s) => self.dest_path_text = s,
            Message::GeneratePressed => {
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

                let options = &CourseListOptions {
                    show_price: self.show_price,
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
                        src_path: self.src_path_text.as_str().into(),
                        src_sheet: self.src_sheet_text.as_str().into(),
                        src_column: self.src_column_text.as_str().into(),
                        dest_path: self.dest_path_text.as_str().into(),
                    }),
                    |result| Message::StorePreferences(result.map_err(|err| format!("{}", err))),
                );
            }
            Message::BackPressed => match self.state {
                State::Error | State::Result => self.state = State::Entry,
                _ => {}
            },
            Message::ShowPriceToggled(b) => self.show_price = b,
            Message::LoadPreferences(prefs) => {
                self.src_path_text = prefs.src_path.to_string();
                self.src_sheet_text = prefs.src_sheet.to_string();
                self.src_column_text = prefs.src_column.to_string();
                self.dest_path_text = prefs.dest_path.to_string();
            }
            Message::StorePreferences(result) => {
                if let Err(err) = result {
                    error!("Could not store preferences: {}", err)
                }
            }
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let column = Column::new()
            .align_items(Align::Center)
            .padding(10)
            .spacing(10);

        match self.state {
            State::Entry => column
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
                    Row::new()
                        .align_items(Align::Start)
                        .padding(20)
                        .spacing(10)
                        .push(Checkbox::new(
                            self.show_price,
                            "Show Price",
                            Message::ShowPriceToggled,
                        )),
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
                ),
            State::Error => column
                .push(Row::new().push(Text::new(self.error_text.clone()).size(18)))
                .push(Space::with_height(Length::Fill))
                .push(
                    Row::new().align_items(Align::Start).push(
                        Button::new(&mut self.back_button, Text::new("Ok"))
                            .on_press(Message::BackPressed),
                    ),
                ),
            State::Result => column
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
