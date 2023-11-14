

![preview](https://github.com/imsnif/monocle/assets/795598/67c0332c-cc05-4e59-88fa-b7f9d5e9472a)

## About
This [Zellij][zellij] plugin is a fuzzy finder for file names and their contents.

It can open results in your `$EDITOR` (scrolled to the correct line), as floating or tiled panes.

It can open a new terminal pane to the location of the file, as a floating or tiled pane.

It will ignore hidden files and respect your `.gitignore`. If you press `ESC` or `Ctrl c`, it will hide itself until you call it again.

[zellij]: https://github.com/zellij-org/zellij

## Try it out

From inside Zellij:
```
zellij plugin -- https://github.com/imsnif/monocle/releases/latest/download/monocle.wasm
```

## Permanent Installation
1. Download the `monocle.wasm` file from the latest release
2. Place it in `~/.config/zellij/plugins`
3. From inside Zellij, run `zellij plugin [--floating] [--in-place] -- file:~/zellij/plugins/monocle.wasm`

## Kiosk Mode
Monocle can be stared in "Kiosk Mode" - meaning that it will open files on top of itself instead of in a new pane. This can be especially powerful when combined with opening the monocle plugin itself "in-place".

Example:
```
zellij plugin --configuration kiosk=true --in-place -- file:~/.config/zellij/plugins/monocle.wasm
```

## How do I invoke monocle with a keybinding?
Add the following to your [zellij config](https://zellij.dev/documentation/configuration.html) somewhere inside the [`keybinds`](https://zellij.dev/documentation/keybindings.html) section:
```kdl
// bind F1 to open monocle in a new floating pane and open any results in a new tiled/floating pane
bind "F1" {
    LaunchOrFocusPlugin "file:~/.config/zellij/plugins/monocle.wasm" {
        floating true
    };
    SwitchToMode "Normal"
}
// bind "Alt m" to open monocle on top of the current pane and open any results on top of itself
bind "Alt m" {
    LaunchPlugin "file:~/.config/zellij/plugins/monocle.wasm" {
        in_place true
        kiosk true
    };
    SwitchToMode "Normal"
}
```

## Development

Load the `dev.kdl` layout from inside zellij: `zellij action new-tab -l dev.kdl` or from outside Zellij with `zellij -l dev.kdl`

## Known issue
Does not like (read: crashes on) device files and other non-file entities. This is an upstream issue with the Zellij version of wasmer. It is recommended not to use it inside system folders.
