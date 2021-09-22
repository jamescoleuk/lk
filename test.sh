#!/usr/bin/env bash
#
# This is a comment on the file.
# Here's a second line.

# This function is very clever and awesome and does a lot of neat stuff.
# And here is some more detailed description about this funciton. Isn't it great?
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
    _hidden_function
}

_hidden_function() {
    echo "blah blah"
}

cargo run $(basename "$0") "$@" || "$@"
