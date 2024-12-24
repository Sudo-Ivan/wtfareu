use iced::widget::{button, container, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use serde_json::Value;
use std::process::Command as ProcessCommand;
use iced::subscription::events_with;
use iced::Event;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_millis(100);

pub fn main() -> iced::Result {
    WtfAreU::run(Settings {
        window: iced::window::Settings {
            size: (420, 600),
            ..Default::default()
        },
        antialiasing: false,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    WindowClicked(String),
    Refresh,
}

struct WtfAreU {
    windows: Vec<WindowInfo>,
    last_update: Instant,
    cached_windows: Option<Vec<WindowInfo>>,
}

#[derive(Debug, Clone)]
struct WindowInfo {
    title: String,
    workspace: String,
    app_class: String,
}

impl Application for WtfAreU {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            WtfAreU { 
                windows: Vec::new(),
                last_update: Instant::now(),
                cached_windows: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Where The Flip Are U")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::WindowClicked(workspace) => {
                let _ = ProcessCommand::new("hyprctl")
                    .args(["dispatch", "workspace", &workspace])
                    .output();
                self.cached_windows = None; // Invalidate cache on workspace change
                Command::none()
            }
            Message::Refresh => {
                // Only refresh if cache is invalid
                if self.cached_windows.is_none() || self.last_update.elapsed() > CACHE_DURATION {
                    self.windows = self.get_hyprland_clients();
                    self.last_update = Instant::now();
                    self.cached_windows = Some(self.windows.clone());
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let windows = if let Some(cached) = &self.cached_windows {
            cached
        } else {
            &self.windows
        };
        
        let grouped = self.group_windows(windows);
        
        // Preallocate vector capacity
        let mut workspace_nums = Vec::with_capacity(grouped.len());
        workspace_nums.extend(
            grouped.keys()
                .filter_map(|w| w.parse::<i32>().ok())
        );
        workspace_nums.sort_unstable(); // Use sort_unstable for better performance

        let workspace_columns = workspace_nums.iter()
            .map(|num| {
                let workspace = num.to_string();
                let windows = grouped.get(&workspace).unwrap();
                
                Column::new()
                    .spacing(2)
                    .push(
                        text(format!("Workspace #{}", workspace))
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                                0.533, 0.752, 0.816
                            )))
                    )
                    .push(
                        windows.iter().fold(
                            Column::new().spacing(2),
                            |column, window| {
                                column.push(
                                    button(
                                        row![
                                            text(&format!("{} ({})", window.title, window.app_class))
                                                .width(Length::Fill)
                                        ]
                                        .spacing(10)
                                        .padding(iced::Padding::from([0, 0, 0, 15]))
                                    )
                                    .padding(10)
                                    .style(iced::theme::Button::Custom(Box::new(DarkButton)))
                                    .width(Length::Fill)
                                    .on_press(Message::WindowClicked(window.workspace.clone()))
                                )
                            }
                        )
                    )
            })
            .fold(Column::new().spacing(10), |column, workspace_column| {
                column.push(workspace_column)
            });

        let window_list = scrollable(workspace_columns)
            .height(Length::Fill);

        container(window_list)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Custom(Box::new(DarkContainer)))
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        events_with(|event, _| match event {
            Event::Window(_) => Some(Message::Refresh),
            _ => None,
        })
    }
}

impl WtfAreU {
    fn get_hyprland_clients(&self) -> Vec<WindowInfo> {
        let output = ProcessCommand::new("hyprctl")
            .args(["-j", "clients"])
            .output();

        match output {
            Ok(output) => {
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(windows) = json.as_array() {
                        let mut result = Vec::with_capacity(windows.len());
                        for window in windows {
                            if let (Some(title), Some(workspace), Some(app_class)) = (
                                window.get("title").and_then(|v| v.as_str()),
                                window.get("workspace").and_then(|v| v.get("name")).and_then(|v| v.as_str()),
                                window.get("class").and_then(|v| v.as_str()),
                            ) {
                                result.push(WindowInfo {
                                    title: title.to_string(),
                                    workspace: workspace.to_string(),
                                    app_class: app_class.to_string(),
                                });
                            }
                        }
                        return result;
                    }
                }
            }
            Err(_) => {}
        }
        Vec::new()
    }

    fn group_windows<'a>(&self, windows: &'a [WindowInfo]) -> HashMap<String, Vec<&'a WindowInfo>> {
        let mut groups = HashMap::with_capacity(10); // Preallocate with reasonable capacity
        for window in windows {
            groups.entry(window.workspace.clone())
                .or_insert_with(|| Vec::with_capacity(5))
                .push(window);
        }
        groups
    }
}

struct DarkContainer;

impl container::StyleSheet for DarkContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.18, 0.204, 0.251,
            ))),
            border_radius: 5.0.into(),
            ..Default::default()
        }
    }
}

struct DarkButton;

impl button::StyleSheet for DarkButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.18, 0.204, 0.251,
            ))),
            border_radius: 5.0.into(),
            text_color: iced::Color::WHITE,
            shadow_offset: iced::Vector::new(0.0, 0.0),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.23, 0.26, 0.32,
            ))),
            ..active
        }
    }
}
