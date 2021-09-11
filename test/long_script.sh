#!/usr/bin/env bash

# This function is very clever and awesome and does a lot of neat stuff.
# And here is some more detailed. wefsdjfklsdjhf
some_function() {
    echo "hello world from a script"
    echo "foobar"
    sleep 1
    echo "ending function now"
}

another_function() {
    echo "hello from another function"
}

# More functions
yet_more_functions() {
    echo "hello from another function"
}

# This function has a stupidly long name and it won't render that well.
really_really_really_really_really_really_long_function_name() {
    echo "hello from another function"
}

runsh $(basename "$0") "$@" || "$@"
