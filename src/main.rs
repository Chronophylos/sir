use anyhow::Result;
use iced::{
    button, text_input, window, Align, Button, Column, Element, Length, Row, Sandbox, Settings,
    Space, Text, TextInput,
};

fn main() -> Result<()> {
    //let path = format!("{}/data/sir.xlsx", env!("CARGO_MANIFEST_DIR"));
    //get_courses_participans(path)?;

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
    OkPressed,
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
    ok_button: button::State,

    error_text: String,
    result_text: String,

    state: State,
}

impl Sandbox for Main {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SrcPathInputChanged(s) => self.src_path_text = s,
            Message::SrcSheetInputChanged(s) => self.src_sheet_text = s,
            Message::SrcColumnInputChanged(s) => self.src_column_text = s,
            Message::DestPathInputChanged(s) => self.dest_path_text = s,
            Message::GeneratePressed => {
                let table = match sir::read_course_list(
                    &self.src_path_text,
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

                let groups = table.keys().count();
                let participants = table.values().map(|v| v.len()).fold(0, |acc, x| acc + x);

                match sir::write_course_list(&self.dest_path_text, table) {
                    Ok(_) => {}
                    Err(err) => {
                        self.error_text = format!("Error writing courses: {:?}", err);
                        self.state = State::Error;
                        return;
                    }
                }

                self.result_text = format!(
                    "Found {} groups and {} participants.\nWrote result to {}",
                    groups, participants, &self.dest_path_text
                );

                self.state = State::Result;
            }
            Message::OkPressed => self.state = State::Entry,
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
                        .align_items(Align::Center)
                        .padding(20)
                        .spacing(10)
                        .push(Text::new("Destination"))
                        .push(
                            TextInput::new(
                                &mut self.dest_path_input,
                                "path to worksheet",
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
                .push(Text::new(self.error_text.clone()).size(18))
                .push(Space::with_height(Length::Fill))
                .push(
                    Button::new(&mut self.ok_button, Text::new("Ok")).on_press(Message::OkPressed),
                ),
            State::Result => column.push(Text::new(self.result_text.clone()).size(20)),
        }
        .into()
    }
}
