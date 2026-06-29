use iced::widget::{column, container, text_input};
use iced::{Alignment, Element, Length, Subscription, Task};
use rand::seq::index;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

const LOWER_CASE:&[u8] = "abcdefghjkmnpqrstuvwxyz".as_bytes();
const UPPER_CASE:&[u8] = "ABCDEFGHJKMNPQRSTUVWXYZ".as_bytes();
const NUMERIC:&[u8] = "23456789".as_bytes();
const SPECIAL:&[u8] = "!@#$%*?:".as_bytes();

pub fn main() -> iced::Result {
    iced::application(PopupApp::default, PopupApp::update, PopupApp::view)
        .window(iced::window::Settings {
            size: (200.0, 100.0).into(),
            ..Default::default()
        })
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
            #[cfg(target_os = "linux")]
            gtk::init().expect("Failed to initialize GTK on tray thread");

            let tray_menu = Menu::new();
            let generate_item = MenuItem::new("Generate Password", true, None);
            let _ = tray_menu.append(&generate_item);
            let exit_item = MenuItem::new("Exit", true, None);
            let _ = tray_menu.append(&exit_item);

            let icon = load_icon();
            
            // Build the StatusNotifierItem
            let _tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_icon(icon)
                .with_tooltip("Iced Generator")
                .build()
                .unwrap();

            // Run a mini GTK event loop iteration explicitly to process D-Bus registration
            #[cfg(target_os = "linux")]
            loop {
                gtk::main_iteration();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }

            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                let mut msg = std::mem::zeroed();
                // This loop blocks the thread and processes Win32 events
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        
        });

        Self {
            text_content: generate_password(),
        }
    }
}

impl PopupApp {
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

    fn view(&self) -> Element<'_, Message> {
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

fn load_icon() -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image_bytes = include_bytes!("../resources/images/password_32.png");
        let image = image::load_from_memory(image_bytes)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn generate_password() -> String {
    let mut pwd:Vec<u8> = Vec::new();
    let num_special: usize = 2;
    let num_numeric = rand::random_range(3..7) as usize;
    let num_upper = rand::random_range(3..9) as usize;
    let num_lower = 20 - num_upper - num_numeric - num_special;
    let mut rng = rand::rng();
    let chars:Vec<_> = index::sample(&mut rng, LOWER_CASE.len(), num_lower).iter()
        .map(|i| LOWER_CASE[i])
        .collect();
    pwd.extend(chars);
    let chars:Vec<_> = index::sample(&mut rng, UPPER_CASE.len(), num_upper).iter()
        .map(|i| UPPER_CASE[i])
        .collect();
    pwd.extend(chars);
    let chars:Vec<_> = index::sample(&mut rng, NUMERIC.len(), num_numeric).iter()
        .map(|i| NUMERIC[i])
        .collect();
    pwd.extend(chars);
    let chars:Vec<_> = index::sample(&mut rng, SPECIAL.len(), num_special).iter()
        .map(|i| SPECIAL[i])
        .collect();
    pwd.extend(chars);

    String::from_utf8(pwd).unwrap()
}

