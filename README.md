# cyrs

A simple C-c C-v tool in command line.

Inspired by `MACOS` built-in **copy**(`⌘c`) **paste**(`⌘⌥c`) **move**(`⌘⌥v`).

> ⚠️ WORK IN PROGRESS.

## Install

``` bash
cargo install cyrs
```

## Usage

```
cyrs 0.2.1
ycycwx <yytcjcy@gmail.com>
A simple C-c C-v tool in command line.

USAGE:
    cy [INPUT]...
    cy <SUBCOMMAND>

ARGS:
    <INPUT>...    Mark files into clipboard

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add      Add files into clipboard [aliases: a]
    copy     Copy all files in clipboard to target dir [aliases: c, cp]
    help     Print this message or the help of the given subcommand(s)
    list     List all files in clipboard [aliases: l, ls, show]
    move     Move all files in clipboard to target dir [aliases: m, mv]
    reset    Reset clipboard [aliases: clear]
```
