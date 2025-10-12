use anyhow::Result;
use iced::{
    ContentFit, Length, Subscription, Task,
    widget::{Column, Row, button, column, horizontal_rule, image, row, text},
    window,
};
use iced_aw::card;
use rfd::FileDialog;
use std::{path::PathBuf, sync::Arc};
use tracing::{debug, error};
use tracing_subscriber::{filter::EnvFilter, fmt::Subscriber};

pub mod ipc;
pub mod ipc_spec;

use ipc::*;
use ipc_spec::*;

#[derive(Default, PartialEq)]
enum Page {
    #[default]
    Connect,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
enum Message {
    Connect,
    Connected(Arc<IpcHandle>),
    ConnectionFailed(String),
    SelectMonitor, // TODO save monitors in context and select from them with index or smh
    SelectFileForBackground,
    SelectedFileForBackground(Option<PathBuf>),
    SendBackgroundOptions,
    Disconnect,
    Disconnected,
}

#[derive(Default)]
struct BackgroundOpts {
    path: PathBuf,
    // when I'll add support for assiging a singe bakcground to multiple monitors ill have to
    // change this to a vec or smh
    monitor: Option<i8>, // if u have more than 128 monitors hit me up
}

#[derive(Default)]
struct App {
    page: Page,
    user_error: Option<String>,
    ipc_handle: Option<Arc<IpcHandle>>,
    background_opts: BackgroundOpts,
}

// TODO: closed events - https://docs.rs/iced/latest/iced/window/fn.close_events.html

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect => match self.ipc_handle {
                Some(_) => {
                    error!("Can't create a new connection, already connected!");
                    Task::none()
                }
                None => {
                    self.set_page(Page::Connecting);
                    Task::perform(IpcHandle::new(IPC_PATH), |res| match res {
                        Ok(ipc_handle) => Message::Connected(Arc::new(ipc_handle)),
                        Err(err) => Message::ConnectionFailed(format!("{:?}", err)),
                    })
                }
            },
            Message::ConnectionFailed(err) => {
                error!("Failed to connect: {:?}", err);
                self.user_error = Some(err);
                Task::none()
            }
            Message::Connected(ipc_handle) => {
                debug!("Connected to server!");
                self.ipc_handle = Some(ipc_handle);
                self.set_page(Page::Connected);

                Task::none()
            }
            Message::SelectMonitor => Task::none(),
            Message::SelectFileForBackground => {
                debug!("Selecting a background file...");
                Task::perform(
                    async move {
                        FileDialog::new()
                            .set_directory("~")
                            .pick_file()
                            .map(|file| file.to_path_buf())
                    },
                    Message::SelectedFileForBackground,
                )
            }
            Message::SelectedFileForBackground(path) => {
                if let Some(path_ok) = path {
                    self.background_opts.path = path_ok;
                    debug!(
                        "Background file selected: `{}`",
                        self.background_opts.path.to_str().unwrap_or_default()
                    );
                }
                Task::none()
            }

            Message::SendBackgroundOptions => Task::none(),
            Message::Disconnect => match &self.ipc_handle {
                Some(ipc_handle) => {
                    let ipc_clone = ipc_handle.clone();
                    Task::perform(async move { ipc_clone.close().await }, |_| {
                        Message::Disconnected
                    })
                }
                None => {
                    error!("Can't disconnect, connected to nothing!");
                    Task::none()
                }
            },
            Message::Disconnected => {
                self.ipc_handle = None;
                self.set_page(Page::Connect);
                Task::none()
            }
        }
    }

    async fn build_monitors_widgets(&self) -> Row<'_, Message> {
        let mut monitors_widgets = Row::new();
        if let Some(ipc_handle) = self.ipc_handle.as_ref() {
            for monitor in ipc_handle.get_monitors().await {
                monitors_widgets = monitors_widgets.push(
                    button(text!("{}", monitor.index).center())
                        .width(Length::Fill)
                        .on_press(Message::SelectMonitor),
                );
            }
        }
        monitors_widgets
    }

    // TODO: stuff like this:

    // fn build_monitors_widgets_subscription(&self) -> Subscription<_> {
    //     Subscription::run(self.build_monitors_widgets())
    // }

    fn set_page(&mut self, page: Page) {
        self.page = page;
        self.user_error = None
    }

    fn view(&self) -> Column<Message> {
        match self.page {
            Page::Connect => column![
                image("res/logo.webp").content_fit(ContentFit::Cover),
                button("Connect")
                    .width(Length::Fill)
                    .on_press(Message::Connect),
            ]
            .push_maybe(
                self.user_error
                    .as_ref()
                    .map(|e| column![horizontal_rule(50), text(e)]),
            )
            .padding(20),
            Page::Connecting => column![text!["Connecting..."]]
                .push_maybe(
                    self.user_error
                        .as_ref()
                        .map(|e| column![horizontal_rule(50), text(e)]),
                )
                .padding(20),
            Page::Connected => column![
                image("res/logo.webp").content_fit(ContentFit::Cover),
                button("Select file")
                    .width(Length::Fill)
                    .on_press(Message::SelectFileForBackground),
                card(
                    text!["Monitors"],
                    row![
                        button(text("1").center())
                            .width(Length::Fill)
                            .on_press(Message::SelectMonitor),
                        button(text("2").center())
                            .width(Length::Fill)
                            .on_press(Message::SelectMonitor),
                        button(text("3").center())
                            .width(Length::Fill)
                            .on_press(Message::SelectMonitor),
                    ]
                ),
                button("Disconnect")
                    .width(Length::Fill)
                    .on_press(Message::Disconnect),
            ]
            .push_maybe(
                self.user_error
                    .as_ref()
                    .map(|e| column![horizontal_rule(50), text(e)]),
            )
            .padding(20),
        }
    }
    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn main() -> Result<()> {
    // only enable debug for our crate
    Subscriber::builder()
        .with_env_filter(EnvFilter::new("xab_gui=debug"))
        .init();

    debug!("Initializing iced application");
    iced::application("xab gui", App::update, App::view)
        .window(window::Settings {
            ..window::Settings::default()
        })
        .theme(App::theme)
        .run()?;
    debug!("bye");
    Ok(())
}
