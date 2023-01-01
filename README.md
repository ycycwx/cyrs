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
A simple C-c C-v tool in command line.

Usage: cy [INPUT]...
       cy <COMMAND>

Commands:
  add    Add files into clipboard [aliases: a]
  copy   Copy all files in clipboard to target dir [aliases: c, cp]
  move   Move all files in clipboard to target dir [aliases: m, mv]
  list   List all files in clipboard [aliases: l, ls, show]
  reset  Reset clipboard [aliases: clear]
  help   Print this message or the help of the given subcommand(s)

Arguments:
  [INPUT]...  Mark files into clipboard

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
