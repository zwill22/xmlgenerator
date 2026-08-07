#!/bin/sh

echo "Setup act environment"
apt-get update
echo "Installing cmake, rustup, and clang"
apt-get install cmake rustup clang -y -q

echo "Installing rustup toolchain"
rustup default stable
echo "Setup complete"
