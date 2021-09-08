## Challenges

Executing a bash function from rust:
1. `Command` executes the program directly and does not create a shell, so you can't source a script and then invoke it.
2. We could use `shellfn` to source and execute but you lose the ability to see what the script is doing as it runs. This has to support long-running scripts, so that isn't any good.

It's easier to have the script invoke itself, and that's what the last line does. This means runsh doesn't actually run anything, it's just a pretty-printer for bash scripts. I'd rather have it run the functions becuase I want to keep the bash simple, so if anyone reading this has any better ideas please do get in touch.

## Why not run_lib

A Rust executable is easier to distribute via cargo. It's easier for people to update. Integration with a script is more or less the same. The processing is much easier in Rust than it is in bash, i.e. finding and displaying multi-line comments. 

## How the integration works
Integration looks like this:
```runsh $(basename "$0") "$@" || "$@"```

There are two parts to this:
1. `runsh $(basename "$0") "$0"`: this executes runsh, passing two parameters: the name of the script being run (`$(basename "$0")`) and the parameters to the shell command as issued by the user (`"$@"`). This will be the function name and args, e.g. in `./script.sh some_function` it will be `some_function`.
2. `|| "$@"` is the fall back for when runsh returns a non-zero exit code. It'll return a non-zero exit code when it has identified the function being run, i.e. when it's validated the function as actually exisitng in the target script. So when this happens `"$@"` will execute, which is a very quick way of running the actual function.