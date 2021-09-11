# runsh

A CLI frontend for your bash scripts. 

Parses scripts and pretty prints the functions it finds. Similar to [run_lib](https://github.com/jamescoleuk/run_lib) but rustier.

## Installation
`cargo install runsh`

### Update
```bash
cargo uninstall runsh
cargo install runsh
```

## Use
Add the following to the end of your script:
```runsh $(basename "$0") "$@" || "$@"```

## Challenges

Executing a bash function from rust:
1. `Command` executes the program directly and does not create a shell, so you can't source a script and then invoke it.
2. We could use `shellfn` to source and execute but you lose the ability to see what the script is doing as it runs. This has to support long-running scripts, so that isn't any good.

It's easier to have the script invoke itself, and that's what the last line does. This means runsh doesn't actually run anything, it's just a pretty-printer for bash scripts. I'd rather have it run the functions becuase I want to keep the bash simple, so if anyone reading this has any better ideas please do get in touch.

## Why not run_lib?

I already wrote this in bash and called it [run_lib](https://github.com/jamescoleuk/run_lib). There are a few reasons why this is better. Here are some considerations:
1. A Rust executable is easier to distribute via `cargo`. It's easier for people to update. 
2. Integration with a script is more or less the same. 
3. The processing is much easier in Rust than it is in bash, i.e. finding and displaying multi-line comments. 
4. Rust so hot right now.

## How the integration works
Integration looks like this:
```runsh $(basename "$0") "$@" || "$@"```

There are two parts to this:
1. `runsh $(basename "$0") "$0"`: this executes runsh, passing two parameters: 
   1. The name of the script being run (`$(basename "$0")`). E.g. in `./script som_function` it will be `script`.
   2. The parameters to the shell command as issued by the user (`"$@"`). This will be the function name and args, e.g. in `./script.sh some_function some_args` it will be `some_function some_args`.
2. `|| "$@"` is the fall back for when `runsh` returns a non-zero exit code. This is suppsoed to happen and how the function ends up getting run. `runsh` will validate the function name and return a non-zero exit code if it exists. When this happens `"$@"` will execute, which is a quick bash way to run the actual function.