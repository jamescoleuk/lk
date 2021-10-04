# lk

A CLI frontend for your bash scripts. 

Parses scripts and pretty prints the functions it finds. Similar to [run_lib](https://github.com/jamescoleuk/run_lib) but rustier, and to [runsh](https://github.com/jamescoleuk/runsh) but better.

Say you have a script called `script.sh` that looks like this:

```
#!/usr/bin/env bash

# This function is very clever and awesome and does a lot of neat stuff.
# And here is some more detailed description about this funciton. Isn't it great?
some_function() {
    echo "hello world from a script"
    echo "foobar"
    sleep 1
    echo "ending function now"
}

# More functions
yet_more_functions() {
    echo "hello from another function"
}
```

You can access it by executing `lk`, and it'll find the script and should you what functions are available. Then you can run something like this to execute the function:
```bash
lk script.sh some_function
```

## Why "lk"
This is a tool that I use a lot, and "lk" is short and ergonomic. As long as you're reasting on the home keys.

## Installation
From [the crate](https://crates.io/crates/lk):
```bash
cargo install lk
```

### Update
```bash
cargo install --force lk
```

## Use
Just execute `lk` and follow the instructions.


### File headers
`lk` will extract comments in the file header, if it finds any, and display them alongside all your runnable functions. It relies on these comments following the form in the [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html#s4.1-file-header), i.e. like this:
```bash
#!/usr/bin/env bash
#
# Some comments.
# And some more.
```

## Why not run_lib?

I already wrote this in bash and called it [run_lib](https://github.com/jamescoleuk/run_lib). There are a few reasons why this might be better. Here are some considerations:
1. A Rust executable is easier to distribute via `cargo`. It's easier for people to update their version. 
2. Integration with a script is more or less the same. 
3. The processing is much easier in Rust than it is in bash, i.e. finding and displaying multi-line comments. 
4. Rust so hot right now.

## Finding bash files
### Testing for binaries
We don't want binaries because we won't be reading any functions from them.

For testing I took the smallest binary from my `/usr/bin` and copied it into `./tests/`

## TODO

- [ ] Test for bash files. At the moment it will attemp to display any executable text file.
- [ ] Add syntax to allow functions to be ignored, e.g. an `_` prefix.
- [ ] Don't return a non-0 exit code when exploring scripts, i.e. running without valid script and function arguments. This is a hangup from `runsh`, where it used non-0 exit codes to run the scripts.