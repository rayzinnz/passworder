use iced::widget::{button, column, container, text_input};
use iced::{Alignment, Element, Length, Subscription, Task, keyboard, window};
use rand::seq::index;
use tray_icon::{
    event::{Event, EventReceiver},
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};

const LOWER_CASE:&[u8] = "abcdefghjkmnpqrstuvwxyz".as_bytes();
const UPPER_CASE:&[u8] = "ABCDEFGHJKMNPQRSTUVWXYZ".as_bytes();
const NUMERIC:&[u8] = "23456789".as_bytes();
const SPECIAL:&[u8] = "!@#$%*?:".as_bytes();

pub fn main() -> iced::Result {
    iced::application(PopupApp::default, PopupApp::update, PopupApp::view)
        .window(window::Settings {
            size: (300.0, 200.0).into(),
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
    EscPressed,
    Exit,
    TrayMenuClicked(String),
    TextChanged(String),
}

impl Default for PopupApp {
    fn default() -> Self {
        Self {
            text_content: generate_password(),
        }
    }
}

impl PopupApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EscPressed => {
                // Hide the window when ESC is pressed
                return window::latest()
                    .and_then(|id| window::set_mode(id, window::Mode::Hidden));
            }
            Message::Exit => {
                return window::latest().and_then(window::close);
            },
            Message::TrayMenuClicked(_id) => {
                // Show the window and then bring it to the front
                return window::latest()
                    .and_then::<Option<window::Id>>(|id| window::set_mode(id, window::Mode::Windowed))
                    .and_then(|id| window::gain_focus(id));
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
            column![
                input,
                button("Exit").padding([10, 20]).on_press(Message::Exit),
                ]
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
        fn handle_hotkey(event: keyboard::Event) -> Option<Message> {
            use keyboard::key;

            let keyboard::Event::KeyPressed { modified_key, .. } = event else {
                return None;
            };

            match modified_key.as_ref() {
                keyboard::Key::Named(key::Named::Escape) => Some(Message::EscPressed),
                _ => None,
            }
        }

        let tray_sub = iced::subscription::channel(
            std::time::Duration::from_millis(16),
            100,
            |mut output| async move {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                
                std::thread::spawn(move || {
                    #[cfg(target_os = "linux")]
                    gtk::init().expect("Failed to initialize GTK on tray thread");

                    let tray_menu = Menu::new();
                    let _generate_item = MenuItem::new("Generate Password", true, None);
                    let _ = tray_menu.append(&_generate_item);
                    let exit_item = MenuItem::new("Exit", true, None);
                    let _ = tray_menu.append(&exit_item);
                    let exit_id = exit_item.id();

                    let icon = load_icon();
                    let _tray_icon = TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu))
                        .with_icon(icon)
                        .with_tooltip("Iced Generator")
                        .build()
                        .unwrap();

                    let event_receiver = EventReceiver::new();

                    loop {
                        #[cfg(target_os = "linux")]
                        gtk::main_iteration();

                        #[cfg(target_os = "windows")]
                        unsafe {
                            use windows::Win32::UI::WindowsAndMessaging::*;
                            let mut msg = std::mem::zeroed();
                            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE.0) {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }

                        if let Some(event) = event_receiver.poll_iter().next() {
                            if let Event::MenuItemClick { id, .. } = event {
                                if id == exit_id {
                                    let _ = tx.send(Message::Exit);
                                }
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(16));
                    }
                });

                while let Some(msg) = rx.recv().await {
                    let _ = output.send(msg).await;
                }
            },
        );

        Subscription::batch(vec![
            keyboard::listen().filter_map(handle_hotkey),
            tray_sub,
        ])
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
