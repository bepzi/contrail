# contrail

Highly configurable shell prompter, inspired
by [powerline-shell](https://github.com/banga/powerline-shell)
and [bash-powerline](https://github.com/riobard/bash-powerline).

Customizations are made via a TOML file.

![In action](examples.main.png)

**Disclaimer:** A large part of this program's functionality depends
on the user's shell, font, and terminal emulator. Each shell and
terminal emulator supports different effects. Therefore, there may be
some options that *do not work* with your setup! Weird/missing symbols
can be fixed by changing the config or your terminal's font.

Works on BASH and ZSH shells.

Expect frequent breaking changes.

## Installation

You need the latest stable version of
[Rust](https://www.rust-lang.org) (install with
[rustup](http://doc.crates.io/index.html)).

Ensure your `$PATH` includes `$HOME/.cargo/bin`.

Clone the repository and install with `cargo`:

```bash
git clone https://github.com/ben01189998819991197253/contrail ~/contrail
cd ~/contrail
cargo test && cargo install
# If updating, you may need to do `cargo install --force`
```

### BASH

In your `~/.bashrc`:

```bash
# ~/.bashrc
ps1() {
    PS1="$(contrail -e $? --config $HOME/path/to/config.toml) "
}

PROMPT_COMMAND="ps1; $PROMPT_COMMAND"
```

### ZSH

In your `~/.zshrc`

```bash
# ~/.zshrc
precmd() {
    PS1="$(contrail -e $? --shell "zsh" --config $HOME/path/to/config.toml) "
}
```

Restart/re-launch your terminal emulator. You'll know if it's working
correctly.

`contrail -h` and `contrail -V` will print the help information and
the version number, respectively.

## Configuration

Contrail can be told about the location of the config file with the
`--config` option.

Each part of the prompt is split up into "modules". A typical prompt
might have a "cwd" module (shows the current working directory), a
"git" module (shows the current state of a git repo), and a "prompt"
module (changes color depending on the last exit code).

## Contributing

...is welcomed! Please submit any issues and pull requests, although
do run your code through `rustfmt` first, please.

## Other

#### "Help, when I try to run contrail it just crashes!"

Make **100% sure** your config file is syntactically correct
TOML. Otherwise, file an issue or dig into the source code and try to
fix it :)
