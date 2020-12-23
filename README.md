cp77-keys
=========

*A small utility to generate a new set of keybindings for Cyberpunk 2077*

I use a Dvorak keyboard. Other people use different keyboard layouts. While I don't doubt that eventually you'll be able to rebind every key in Cyberpunk 2077, I don't want to wait. This command line tool transforms the `inputUserMappings.xml` file according to a map (see `maps/dvorak.json` for an example.)

Why a tool, you ask? Because every time the game is updated, the keybindings are lost.

## Installation

Until I get around to setting up a build, you'll have to compile the utility yourself. First, you'll need to [install Rust](https://www.rust-lang.org/tools/install) and download [the source code](https://github.com/tecywiz121/cp77-keys/archive/master.zip) and extract it.

Then, from the extracted source directory, you can do something like:

```
cargo run -- maps/dvorak.json
```

The above command will transform the game's keybindings so that the new bindings are where they would have been on a QWERTY layout. For example, move forward used to be `W` and is now `,`.
