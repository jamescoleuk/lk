#!/usr/bin/env bash
#
# First line of file header comment
# Second line of file header comment

val="foobar"

# This function is very clever and awesome and does a lot of neat stuff.
# And here is some more detailed description about this funciton. Isn't it great?
some_function() {
    echo "hello world from a script"
    echo "foobar"
    sleep 1
    echo "ending function now: ${val}"
}

another_function() {
    echo "hello from another function"
}

# More functions
yet_more_functions() {
    echo "hello from another function"
    _hidden_function
}

printing_function() {
    echo "You said $1 $2"
}

_hidden_function() {
    echo "blah blah"
}
