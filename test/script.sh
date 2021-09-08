#!/usr/bin/env bash

# This function is very clever and awesome and does a lot of neat stuff.
#
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

runsh $(basename "$0") "$@" || "$@"
