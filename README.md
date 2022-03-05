# lk


[![Crates.io](https://img.shields.io/crates/v/lk.svg)](https://crates.io/crates/lk)


A CLI frontend for your bash scripts, focused on ergonomics.

`lk` searches for scripts, parses them and finds bash functions. It can then either:

* let you explore and execute functions, rather like sub-commands in git
* let you fuzzy find and execute functions, similar to the wonderful `fzf`'s `ctrl-r` feature. 

`lk`'s list mode works like this:

![A CLI recording showing how you can use lk's list feature](./docs/how_to_list.svg)

`lk'`s fuzzy mode works like this:
![A CLI recording showing how you can use lk's fuzzy feature](./docs/how_to_fuzzy.svg)

I use both modes, but I default to fuzzy. You can change the default like this:

![A CLI recording showing how you can change lk's default to either list or fuzzy](./docs/how_to_change_default.svg)

## Features 
 - `lk` finds executable non-binary files in the current directory and any sub-directory
 - `lk` finds and displays comment headers from your scripts 
 - `lk` finds and displays comments for functions
 - `lk` ignores functions prefixed with `_`. 
 - `lk` uses a temporary file to execute the script, but you shouldn't need to worry about that
 - If you use fuzzy mode `lk` will write the command you execute to your history

## Installation
From [the crate](https://crates.io/crates/lk):
```bash
cargo install lk
```

## Update
```bash
cargo install --force lk
```

## Use
Just execute `lk` and follow the instructions. `lk --help` is also a thing you can run.

There are lots of ways to write bash and to organise scripts. `lk` might not have encountered them all before. If there's a problem I implore you to raise a bug, or just email me. I will fix it.

## Why?
1. You're a polyglot engineer with package manager fatigue. So you want to hide it all behind some bash, the lingua franca.
2. You do a lot of devops and have a lot of bash.
3. You have a lot of projects that you don't work on for months at a time, and you need to bring some consistency to the experience of re-visiting them.
4. You use `make` and `PHONY` to do non-compile stuff to your project. `lk` just lets your write proper bash without all the `make` specific guff.
5. You ever copy and paste bash from a text file you keep somewhere.

### Use case examples

1. AWS: 
   1. You need to pull down config from AWS and store it in `.env` files.
   2. You need to switch between AWs environments
2. You need to build and deploy many services, and want to hide the edge cases. E.g. for compiling, building, and deploying you might have `lk my_service jfdi`.
3. You regularly need to set up SSH tunneling and can't remember the commands.

## How to write your bash files so they work with lk
Big design goal: you shouldn't have to. But there are many styles of bash, and if `lk` doesn't work with how you write your bash then please let me know and I'll be all over fixing it.

Having said that `lk` does support comments. `lk` will extract comments from file and function headers, if it finds any, and display them alongside all your runnable functions. At the moment it relies on these comments following the form in the [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html#s4.1-file-header). I.e. like this:
```bash
#!/usr/bin/env bash
#
# Some comments.
# And some more.


# A glorious function that does all the things
be_glorious() {
    echo "Ta da!"
}
```

## Configuration and logging
There's no configuration file for `lk`, but it does store logs in `${HOME}/.config/lk`.

## Why the name "lk"?
If you have any typist home key dicipline and if you flap your right hand at the keyboard there's a good chance you'll type 'lk'. So it's short, and ergonomic.

## What's to come?
* Minor UI fixes. It doesn't always behave as I'd like it to.
* Support scripts in other languages, e.g. Python, rust-script, Typescript.
* Disable colours, for the colourblind
* Add a count to `lk --fuzzy`
* Move ignored files to config, i.e. `~/.config/lk/lk.toml`

## Inspiration

I have previously written two similar tools: 
* [run_lib](https://github.com/jamescoleuk/run_lib) - my first draft and written in bash
* [runsh](https://github.com/jamescoleuk/runsh) - my second draft and written in Rust

`run_lib` still has its uses. I've worked in secure environments where I could not have installed a binary. `run_lib` is just a bash script.

[fzf](https://github.com/junegunn/fzf) is wonderful. The `--fuzzy` option in `lk` comes from years of `ctrl-r` fuzzy finding through my shell history with `fzf`. I almost didn't implement this feature because I thought "why bother? fzf has already done it perfectly." Or rather I thought about piping from `lk` to `fzf`. But having the functionality implemented natively is the right thing for `lk`. But you'll notice, perhaps, that the rendering of the fuzzy search in `lk` draws a lot of visual inspiration from `fzf`. `fzf`, I love you.

## Contributing
Contributions are welcome. Thanks to the following for theirs:

* [traxys](https://github.com/traxys)
* [lmburns](https://github.com/lmburns)