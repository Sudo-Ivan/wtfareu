use iced::widget::{button, container, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use serde_json::Value;
use std::process::Command as ProcessCommand;
use iced::subscription::events_with;
use iced::Event;
use std::collections::HashMap;
use iced::widget::tooltip;

pub fn main() -> iced::Result {
    WtfAreU::run(Settings {
        window: iced::window::Settings {
            size: (420, 600),
            ..Default::default()
        },
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
                windows: Vec::new() 
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
                Command::none()
            }
            Message::Refresh => {
                self.windows = self.get_hyprland_clients();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let windows = self.get_hyprland_clients();
        let grouped = self.group_windows(&windows);
        
        // Convert workspace numbers to integers for sorting
        let mut workspace_nums: Vec<_> = grouped.keys()
            .filter_map(|w| w.parse::<i32>().ok())
            .collect();
        workspace_nums.sort();

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
                                let window_button = button(
                                    row![
                                        text(format!("{} ({})", window.title, window.app_class))
                                            .width(Length::Fill)
                                    ]
                                    .spacing(10)
                                    .padding(iced::Padding::from([0, 0, 0, 15]))
                                )
                                .padding(10)
                                .style(iced::theme::Button::Custom(Box::new(DarkButton)))
                                .width(Length::Fill)
                                .on_press(Message::WindowClicked(window.workspace.clone()));

                                let tooltip_text = format!("{}\nClass: {}", window.title, window.app_class);
                                column.push(
                                    tooltip(
                                        window_button,
                                        tooltip_text,
                                        tooltip::Position::Bottom
                                    )
                                    .style(iced::theme::Container::Custom(Box::new(DarkContainer)))
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
                        return windows
                            .iter()
                            .filter_map(|window| {
                                Some(WindowInfo {
                                    title: window.get("title")?.as_str()?.to_string(),
                                    workspace: window.get("workspace")?
                                        .get("name")?
                                        .as_str()?
                                        .to_string(),
                                    app_class: window.get("class")?.as_str()?.to_string(),
                                })
                            })
                            .collect();
                    }
                }
            }
            Err(_) => {}
        }
        Vec::new()
    }

    fn group_windows<'a>(&self, windows: &'a [WindowInfo]) -> HashMap<String, Vec<&'a WindowInfo>> {
        let mut groups: HashMap<String, Vec<&'a WindowInfo>> = HashMap::new();
        for window in windows {
            groups.entry(window.workspace.clone())
                .or_default()
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
