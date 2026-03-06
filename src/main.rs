use iced::Event;
use iced::Subscription;
use iced::event;
use iced::widget::Image;
use iced::widget::image::Handle;
use iced::widget::image::viewer;
use iced::widget::{button, column, container};
use iced::window;
use iced::{Element, Task};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use tracing_subscriber;
mod image_handler;
use image_handler::{ImageMeta, ImageMetaError};

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // TODO CLI COMMANDS
    //let arg_path = std::env::args().nth(1);

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("Lyt")
        .window(window::Settings {
            blur: (true),
            position: window::Position::Specific(iced::Point {
                x: (1000.0),
                y: (500.0),
            }),
            ..Default::default()
        })
        .run()
}

enum Mode {
    View,
    Browse,
}

#[derive(Debug, Clone)]
enum Message {
    Load,
    Loaded(Result<ImageMeta, ImageMetaError>),
    // FileDropped(PathBuf),
    OpenFile,
    FileOpened(Option<PathBuf>),
}

struct App {
    mode: Mode,
    current_image: Option<Handle>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        // TODO CLI COMMANDS
        // let task = std::env::args()
        //     .nth(1)
        //     .map(|path| {
        //         Task::perform(
        //             async move {
        //                 let (tx, rx) = oneshot::channel();
        //                 std::thread::spawn(move || match ImageMeta::new(&path) {
        //                     Ok(img) => info!("Loaded a {:?} image", img),
        //                     Err(e) => error!("{}", e),
        //                 });
        //                 rx.await.unwrap()
        //             },
        //             Message::Loaded,
        //         )
        //     })
        //     .unwrap_or(Task::none());
        (
            Self {
                mode: Mode::View,
                current_image: None,
            },
            Task::none(), //  task,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Load => Task::perform(
                async move {
                    let (tx, rx) = oneshot::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(ImageMeta::new("wallpaper1.jpg"));
                    });
                    rx.await.unwrap()
                },
                Message::Loaded,
            ),

            Message::Loaded(image) => {
                let image = image.unwrap();
                if let (Some(pixles), Some(width), Some(height)) =
                    (image.pixles, image.width, image.height)
                {
                    info!(width = width, height = height, "Image Loaded");
                    self.current_image = Some(Handle::from_rgba(width, height, pixles));
                }
                Task::none()
            }

            // Message::FileDropped(file) => {
            //     println!("hi");
            //     Task::none()
            // }
            Message::OpenFile => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["jpg", "jpeg", "png"])
                        .set_directory("/")
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                Message::FileOpened,
            ),
            Message::FileOpened(Some(path)) => Task::perform(
                async move {
                    let (tx, rx) = oneshot::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(ImageMeta::new(path.to_str().unwrap()));
                    });
                    rx.await.unwrap()
                },
                Message::Loaded,
            ),

            Message::FileOpened(None) => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, _| match event {
            // Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
            Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => match key {
                iced::keyboard::Key::Character(c) => match c.as_str() {
                    "o" => Some(Message::OpenFile),
                    _ => None,
                },

                _ => None,
            },
            _ => None,
        })
    }

    fn view_image(&self) -> Element<Message> {
        if self.current_image.is_none() {
            let logo = Image::new(Handle::from_bytes(
                include_bytes!("../assets/frieren.jpg").to_vec(),
            ))
            .width(200)
            .height(200);

            return container(button(logo).on_press(Message::OpenFile).style(button::text))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into();
        }
        let content: Element<Message> = match &self.current_image {
            Some(h) => container(viewer(h.clone()).content_fit(iced::ContentFit::Contain))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into(),
            None => container("").into(),
        };

        column![content].into()
    }
    fn view(&self) -> Element<Message> {
        match self.mode {
            Mode::View => self.view_image(),
            Mode::Browse => column![button("hello")].into(),
        }
    }
}
