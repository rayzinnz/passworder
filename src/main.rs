use iced::widget::{column, container, text_input};
use iced::{Alignment, Element, Length, Subscription, Task, Theme};
use std::sync::mpsc::{self, Receiver};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

pub fn main() -> iced::Result {
    iced::application(PopupApp::default, PopupApp::update, PopupApp::view)
        .subscription(PopupApp::subscription)
        .run()
}

struct PopupApp {
    text_content: String,
}

#[derive(Debug, Clone)]
enum Message {
    TrayMenuClicked(String),
    TextChanged(String),
}

impl Default for PopupApp {
    fn default() -> Self {
        // Spin up the Tray Icon on an independent OS thread to bypass the GTK init paradox
        std::thread::spawn(|| {
            // Explicitly initialize GTK on this background thread
            gtk::init().expect("Failed to initialize GTK on tray thread");

            let tray_menu = Menu::new();
            let generate_item = MenuItem::new("Generate", true, None);
            let _ = tray_menu.append(&generate_item);

            // Create a blank 32x32 transparent icon spacer
            let icon = Icon::from_rgba(vec![255; 32 * 32 * 4], 32, 32).unwrap();

            // Build the StatusNotifierItem
            let _tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_icon(icon)
                .with_tooltip("Iced Generator")
                .build()
                .unwrap();

            // Run a mini GTK event loop iteration explicitly to process D-Bus registration
            loop {
                gtk::main_iteration();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });

        Self {
            text_content: String::from("Generated text will appear here..."),
        }
    }
}

impl PopupApp {
    fn title(&self) -> String {
        String::from("Iced Generator Popup")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrayMenuClicked(_id) => {
                // Focus/reveal window under Hyprland when menu item clicked
                return iced::window::latest()
                    .and_then(|id| iced::window::gain_focus(id));
            }
            Message::TextChanged(new_text) => {
                self.text_content = new_text;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let input = text_input("Type context here...", &self.text_content)
            .on_input(Message::TextChanged)
            .padding(10);

        container(
            column![input]
                .spacing(10)
                .align_x(Alignment::Center)
                .max_width(380),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            iced::stream::channel(100, |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                use iced::futures::sink::SinkExt;
                loop {
                    // Check if a cross-thread menu event arrived from the tray menu receiver
                    if let Ok(event) = MenuEvent::receiver().recv() {
                        let _ = output.send(Message::TrayMenuClicked(event.id.0)).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            })
        })
    }
}
