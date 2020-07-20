use anyhow::Result;
use iced::{
    button, text_input, window, Align, Button, Checkbox, Column, Element, Length, Row, Sandbox,
    Settings, Space, Text, TextInput,
};
use log::info;
use sir::preferences::{load_preferences, store_preferences, Preferences};

fn main() -> Result<()> {
    env_logger::init();

    info!("Runnging main view");
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
    SrcPathInputChanged(String),
    SrcSheetInputChanged(String),
    SrcColumnInputChanged(String),

    DestPathInputChanged(String),

    GeneratePressed,
    BackPressed,

    ShowPriceToggled(bool),
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
}

impl Sandbox for Main {
    type Message = Message;

    fn new() -> Self {
        let prefs = load_preferences().unwrap();
        Self {
            src_path_text: prefs.src_path.to_string(),
            src_sheet_text: prefs.src_sheet.to_string(),
            src_column_text: prefs.src_column.to_string(),
            dest_path_text: prefs.dest_path.to_string(),
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        format!(
            "SiR Course List Generator Version {}",
            env!("CARGO_PKG_VERSION")
        )
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SrcPathInputChanged(s) => self.src_path_text = s,
            Message::SrcSheetInputChanged(s) => self.src_sheet_text = s,
            Message::SrcColumnInputChanged(s) => self.src_column_text = s,
            Message::DestPathInputChanged(s) => self.dest_path_text = s,
            Message::GeneratePressed => {
                match store_preferences(Preferences {
                    src_path: self.src_path_text.as_str().into(),
                    src_sheet: self.src_sheet_text.as_str().into(),
                    src_column: self.src_column_text.as_str().into(),
                    dest_path: self.dest_path_text.as_str().into(),
                }) {
                    Ok(_) => {}
                    Err(err) => {
                        self.error_text = format!("Error storing preferences: {:?}", err);
                        self.state = State::Error;
                        return;
                    }
                }

                let mut table = match sir::read_course_list(
                    &self.src_path_text,
                    self.show_price,
                    &self.src_sheet_text,
                    &self.src_column_text,
                ) {
                    Ok(table) => table,
                    Err(err) => {
                        self.error_text = format!("Error reading courses: {:?}", err);
                        self.state = State::Error;
                        return;
                    }
                };

                table.sort();

                let participants = table.len();

                match sir::write_course_list(&self.dest_path_text, table) {
                    Ok(_) => {}
                    Err(err) => {
                        self.error_text = format!("Error writing courses: {:?}", err);
                        self.state = State::Error;
                        return;
                    }
                }

                self.result_text = format!(
                    "Found {} participants. Wrote result to {}",
                    participants, &self.dest_path_text
                );

                self.state = State::Result;
            }
            Message::BackPressed => match self.state {
                State::Error | State::Result => self.state = State::Entry,
                _ => {}
            },
            Message::ShowPriceToggled(b) => self.show_price = b,
        }
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
