use std::net::IpAddr;

use iced::widget::{button, column, horizontal_space, row};
use iced::widget::{rich_text, span};
use iced::{font, Color, Element, Font};

use local_ip_address::local_ip;

pub fn main() -> iced::Result {
    iced::application(MainWindow::title, MainWindow::update, MainWindow::view)
        .window_size((300.0, 200.0))
        .centered()
        .run()
}

pub struct MainWindow {
    local_ip: String,
    server_running: bool,
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    StartStopButton,
}

impl MainWindow {
    fn title(&self) -> String {
        String::from("LanKM")
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::StartStopButton => self.server_running = !self.server_running,
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
        let server_button = button(button_text).on_press(Message::StartStopButton);

        let server_row = row![server_status, horizontal_space(), server_button];

        column![ip, server_row].spacing(10).into()
    }
}
