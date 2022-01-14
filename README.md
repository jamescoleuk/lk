# lk

A CLI frontend for your bash scripts, focused on ergonomics.

`lk` searches for scripts, parses them and finds bash functions. It can then either:
* let you explore and execute functions, rather like subcommands in git
* let you fuzzy find and execute functions, similar to the wonderful `fzf`'s `ctrl-r` feature. 

`lk`'s list mode works like this:

![A CLI recording showing how you can use lk's list feature](./docs/how_to_list.svg)
<img src="./docs/how_to_list.svg">

`lk'`s fuzzy mode works like this:
![A CLI recording showing how you can use lk's fuzzy feature](./docs/how_to_fuzzy.svg)
<img src="./docs/how_to_fuzzy.svg">

I use both modes, but I default to fuzzy. You can change the default like this:

![A CLI recording showing how you can change lk's default to either list or fuzzy](./docs/how_to_change_default.svg)
<img src="./docs/how_to_change_default.svg">

## Features 
 - `lk` finds executable non-binary files in the current directory and any sub-directory
 - `lk` finds and displays comment headers from your scripts 
 - `lk` finds and displays comments for functions
 - `lk` ignores functions prefixed with `_`. 
 - `lk` uses a temporary file to execute the script, but you shouldn't need to worry about that

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
* If I end up using `lk --fuzzy` exclusively, as I expect, I'll need to add some way to configure this as a default. That'd make it more useful out-of-the-box.
* Minor UI fixes. It doesn't always behave as I'd like it to.
* Way huge under-the-cover improvements. I have a lot of refactoring to do.
* Support scripts in other languages, e.g. Python, rust-script, Typescript.

## Inspiration

I have previously written two similar tools: 
* [run_lib](https://github.com/jamescoleuk/run_lib) - my first draft and written in bash
* [runsh](https://github.com/jamescoleuk/runsh) - my second draft and written in Rust

`run_lib` still has its uses. I've worked in secure environments where I could not have installed a binary. `run_lib` is just a bash script.


[fzf](https://github.com/junegunn/fzf) is wonderful. The `--fuzzy` option in `lk` comes from years of `ctrl-r` fuzzy finding through my shell history with `fzf`. I almost didn't implement this feature because I thought "why bother? fzf has already done it perfectly." Or rather I thought about piping from `lk` to `fzf`. But having the functionality implemented natively is the right thing for `lk`. But you'll notice, perhaps, that the rendering of the fuzzy search in `lk` draws a lot of visual inspiration from `fzf`. `fzf`, I love you.

## Significant TODOs

- [] Support other script types, e.g. JavaScript, Rust-Script, Python.
- [] `--fuzzy` should write the `--list` command to the prompt, so what happened is in the history. Is this even possible?
- [] Option to disable colours, for the colour blind.
- [] Add a count to --fuzzy -- otherwise how does someone know to scroll up?
- [] We ignore a bunch of stuff: this needs to be in config.