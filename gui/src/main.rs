use std::net::IpAddr;
use std::sync::Arc;

use iced::alignment::Vertical;
use iced::futures::{AsyncReadExt, SinkExt, Stream, StreamExt};
use iced::stream::try_channel;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::widget::{rich_text, span};
use iced::{font, Background, Border, Color, Element, Font, Length, Subscription, Theme};

use local_ip_address::local_ip;

use async_process::{Command, Stdio};

pub fn main() -> iced::Result {
    iced::application(MainWindow::title, MainWindow::update, MainWindow::view)
        .window_size((500.0, 400.0))
        .centered()
        .subscription(MainWindow::subscription)
        .run()
}

pub struct MainWindow {
    local_ip: String,
    server_running: bool,
    command_to_run: Option<Vec<String>>,
    child_log: String,
}

impl Default for MainWindow {
    fn default() -> Self {
        let ip = local_ip().unwrap();
        let local_ip = match ip {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
            }
            IpAddr::V6(_) => {
                todo!()
            }
        };

        Self {
            local_ip,
            server_running: false,
            command_to_run: None,
            child_log: String::new(),
        }
    }
}

fn read_child_output(cmd: Vec<String>) -> impl Stream<Item = Result<String, Error>> {
    try_channel(1, move |mut sender| async move {
        let mut child = Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdout(Stdio::piped())
            .spawn()?;

        loop {
            let output = child.stdout.as_mut().unwrap();
            let mut buffer = [0; 256];
            let n = output.read(&mut buffer).await.unwrap();

            if n == 0 {
                match child.try_status()? {
                    None => { /* Still running, but why the 0? */ }
                    Some(status) => {
                        let s = format!("\nChild exited with code {}\n", status);
                        sender.send(s).await.unwrap();
                        break;
                    }
                }
            }

            let v = Vec::from(&buffer[0..n]);
            let s = String::from_utf8(v).unwrap();
            sender.send(s).await.unwrap();
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
enum Error {
    IoError(Arc<std::io::Error>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(Arc::new(e))
    }
}

#[derive(Debug, Clone)]
enum Message {
    ChildOutput(Result<String, Error>),
    RunChild(Vec<String>),
}

impl MainWindow {
    fn title(&self) -> String {
        String::from("LanKM")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::ChildOutput(s) => match s {
                Ok(s) => {
                    let n = s.replace('\t', "        ");
                    self.child_log.push_str(&n);
                }
                Err(e) => match e {
                    Error::IoError(e) => self.child_log = format!("IO Error: {}", e),
                },
            },
            Message::RunChild(cmd) => {
                self.command_to_run = Some(cmd);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let bold_font = Font {
            weight: font::Weight::Bold,
            ..Font::default()
        };
        let ip = rich_text!["This machine's ip: ", span(&self.local_ip).font(bold_font)].size(16);

        let server_status = if self.server_running {
            span("Running")
                .font(bold_font)
                .color(Color::from_rgb(0.2, 1.0, 0.2))
        } else {
            span("Stopped")
                .font(bold_font)
                .color(Color::from_rgb(1.0, 0.2, 0.2))
        };
        let server_status = rich_text!["Server status: ", server_status];

        let button_text = if self.server_running { "Stop" } else { "Start" };
        let server_button = button(button_text).on_press_with(|| {
            Message::RunChild(vec!["ping".to_string(), "google.com".to_string()])
        });

        let server_row =
            row![server_status, horizontal_space(), server_button].align_y(Vertical::Center);

        let text = text(&self.child_log)
            .color(Color::from_rgb(1.0, 1.0, 1.0))
            .size(14);
        let server_log = scrollable(text)
            .width(Length::Fill)
            .height(Length::Fill)
            .anchor_bottom()
            .style(|theme: &Theme, _| {
                let palette = theme.extended_palette();

                let container = container::Style {
                    text_color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
                    background: Some(Background::Color(palette.background.base.text)),
                    ..container::Style::default()
                };
                let rail = scrollable::Rail {
                    background: None,
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        color: palette.secondary.base.color,
                        border: Border::default(),
                    },
                };
                scrollable::Style {
                    container,
                    vertical_rail: rail,
                    horizontal_rail: rail,
                    gap: None,
                }
            });

        column![ip, server_row, server_log]
            .padding(10)
            .spacing(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.command_to_run {
            Some(cmd) => {
                let x = read_child_output(cmd.clone()).map(Message::ChildOutput);
                Subscription::run_with_id(1, x)
            }
            None => Subscription::none(),
        }
    }
}
