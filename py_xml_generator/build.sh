#!/bin/bash

function reset_conda() {
  if [[ -v OLD_CONDA_PREFIX ]]; then
    export CONDA_PREFIX=${OLD_CONDA_PREFIX}
    unset OLD_CONDA_PREFIX || exit 3
  fi
}

echo "Build py-xmlgenerator"
unset OLD_CONDA_PREFIX
echo "Setup"
if [[ -v CONDA_PREFIX ]]; then
      OLD_CONDA_PREFIX=${CONDA_PREFIX}
      unset CONDA_PREFIX || exit 3
fi

echo "Install dependencies..."
uv sync --all-extras --dev --active --no-install-package pyxmlgenerator || exit 1
uv pip install maturin || exit 2

echo "Build py-xmlgenerator"
uv run --no-project maturin develop --uv
build=$?
if [[ ${build} -ne 0 ]]; then
    echo "Failed to build py-xmlgenerator"
    reset_conda
    exit "${build}"
fi

echo "Build complete"

reset_conda
exit 0
