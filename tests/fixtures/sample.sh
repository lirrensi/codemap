#!/bin/bash

greet() {
    echo "Hello, $1!"
}

add() {
    echo $(($1 + $2))
}

deploy() {
    local env=$1
    echo "Deploying to $env"
}
