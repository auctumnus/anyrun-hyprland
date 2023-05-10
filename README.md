# anyrun-hyprland

Plugin for [anyrun] which allows you to switch between open windows on a running
[Hyprland] instance.

## Build and install

```
$ cargo build --release
$ cp target/debug/libanyrun_hyprland.so ~/.config/anyrun/plugins
```

## Config

The only currently supported config value is `max_entries: usize`.

