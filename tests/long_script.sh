#!/usr/bin/env bash

# More functions
yet_more_functions() {
    echo "hello from another function"
}

# This function has a stupidly long name and it won't render that well.
really_really_really_really_really_really_long_function_name() {
    echo "hello from another function"
}

runsh $(basename "$0") "$@" || "$@"
