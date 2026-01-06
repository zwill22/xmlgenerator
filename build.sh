#!/usr/bin/env bash

function reset_conda() {
  local prefix=$1

  export CONDA_PREFIX=${prefix}
}

function reset_dir() {
  local old_pwd=$1
  cd "${old_pwd}" || exit 4
}

function build() {
  local crate=$1
  local cwd=${PWD}

  cd "${crate}" || return 2

  name=${crate//[_]/}

  echo "Building ${name}"

  echo "Setting up dependencies"
  uv sync --all-extras --dev --active --no-install-package "${name}" || return 1

  echo "Building ${name}"
  uv run maturin develop || return 2

  cd "${cwd}" || return 1

  return 0
}

function build_all() {
    uv pip install pip &> /dev/null
    code=$?

    if [[ ${code} != 0 ]]; then
      echo "Setting up virtual environment"
      uv venv
    fi

    echo "Install maturin"
    uv pip install maturin || return 1

    build "py_xsd_test_data" || return 2

    build "py_xml_generator" || return 2

    return 0
}


# Ensure correct working directory
echo "Running build..."
original_wd=${PWD}

directory=$(dirname "$0")

cd "${directory}" || exit 4

if [[ -v CONDA_PREFIX ]]; then
  echo "Getting conda prefix"
  conda_prefix=${CONDA_PREFIX}
  unset CONDA_PREFIX || exit 3
fi

build_all
code=$?
if [[ ${code} -ne 0 ]]; then
  echo "Build failed"
else
  echo "Build complete"
fi

reset_conda "${conda_prefix}"
reset_dir "${original_wd}"
exit ${code}
