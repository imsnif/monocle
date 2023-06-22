

![preview](https://github.com/imsnif/monocle/assets/795598/67c0332c-cc05-4e59-88fa-b7f9d5e9472a)

## About
This [Zellij][zellij] plugin is a fuzzy finder for file names and their contents.

It can open results in your `$EDITOR` (scrolled to the correct line), as floating or tiled panes.

It can open a new terminal pane to the location of the file, as a floating or tiled pane.

It will ignore hidden files and respect your `.gitignore`. If you press `ESC` or `Ctrl c`, it will hide itself until you call it again.

[zellij]: https://github.com/zellij-org/zellij

## Installation
1. Download the `monocle.wasm` file from the release matching your installed Zellij version
2. Place it in `~/zellij-plugins`
3. From inside Zellij, run `zellij action new-pane --plugin file:~/zellij-plugins/monocle.wasm --floating`

## How do I invoke monocle with a keybinding?
Add the following to your [zellij config](https://zellij.dev/documentation/configuration.html) somewhere inside the [`keybinds`](https://zellij.dev/documentation/keybindings.html) section:
```kdl
shared_except "locked" {
    bind "F1" {
        LaunchOrFocusPlugin "file:~/zellij-plugins/monocle" {
            floating true
        }
    }
}
```

## Development

Load the `dev.kdl` layout from inside zellij: `zellij action new-tab -l dev.kdl` or from outside Zellij with `zellij -l dev.kdl`

## Known issue
Does not deal well with extremely large folders, PRs welcome for smart limitations.
