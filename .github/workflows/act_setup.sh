#!/bin/sh

apt-get update
apt-get install cmake rustup clang -y -q

rustup default stable
