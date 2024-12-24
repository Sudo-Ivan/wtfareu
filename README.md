# Where the flip are u

A personal Hyprland GUI to find out which workspace an app is running in. Uses Iced for the GUI. View apps in their workspaces and click on a app to switch to its workspace.

Uses hyprctl to get the workspace and window information.

**Goals:**

- Low resource usage (as low as we can with Iced)
- Small codebase
- Simple to use

## Dependencies

- Iced = 0.10.0
- Serde_json = 1.0

## Requirements

- Hyprland
- Hyprctl
- Rust 

## Building

```bash
cargo build --release
```
