# XMLGenerator

[![Rust][rust-badge]][rust]
[![Crates.io Version][crates-badge]][crates-repo]
[![Python][python-badge]][python]
[![PyPI Version][pypi-version-badge]][pypi-link]
[![XML][xml-badge]][xml]
[![GitHub][github-badge]][repo]
[![GitHub Release][github-release-badge]][github-release]
[![uv][uv-badge]][uv]
[![Pytest][pytest-badge]][pytest]
[![GitHub Actions][actions-badge]][repo-actions]
[![Test Rust][rust-test-badge]][rust-test]
[![Test Python][python-test-badge]][python-test]
[![Coverage Upload][coverage-upload-badge]][coverage]
[![Read the Docs][rtd-badge]][rtd]
[![Documentation Status][doc-badge]][doc]
[![CodeCov][codecov-badge]][codecov]
[![Coverage][coverage-badge]][coverage]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This project provides a Python package [pyxmlgenerator](py_xml_generator/README.md),
which generates XML instances matching the provided XML Schema (XSD) input.
The package is written in Rust with [PyO3][pyo3] Python bindings allowing direct use from Python.
The `pyxmlgenerator` package is a wrapper for the [xmlgenerator](xml_generator/README.md) Rust crate,
which generates the XML from a given schema.

The project includes several additional Cargo crates to achieve this goal:

- [file_to_string](reader/README.md) - For reading text files
- [xsdvalidator](xsd_validator/README.md) - For validating input XSD
- [regextranslator](regex_translator/README.md) - For translating from XSD pattern style regular expressions to Rust style
- [xsdtestdata](xsd_test_data/README.md) - Test data manager for the [xsdtests] database

Additionally, the project also includes a Python wrapper for the `xsdtestdata` crate - [pyxsdtestdata](py_xsd_test_data/README.md).
This allows the test data to be accessed directly from Python.

## Dependencies

System requirements:

- [Cargo][rust] - Installed via Rustup
- [libxml2-rs] - Rust bindings for the [libxml2][libxml2] C library 
- [Python][python] - Required for Python bindings `pyxmlgenerator`
- [uv][uv] - Recommended for installation of Python packages (Optional)

## Build

To build the Rust `xmlgenerator` library, simply run:

```sh
cargo build
```

To run the Rust test suite, use:

```sh
cargo test
```

To build the Python wrapper `pyxmlgenerator`, run:

```sh
uv sync --package pyxmlgenerator
```

To run the Python test suite, use:

```sh
uv run pytest
```

## Usage

The library can be used either from Python or Rust. Here are examples of each:

### Python

The following example illustrates a similar workflow in Python, using the built-in [argparse](https://docs.python.org/3/library/argparse.html) and [pathlib](https://docs.python.org/3/library/pathlib.html) libraries to read and validate command-line input.

```python
import pathlib
import argparse
import pyxmlgenerator

from xmlschema importXMLSchema

parser = argparse.ArgumentParser()
parser.add_argument("filepaths", type=pathlib.Path, nargs='+')

args = parser.parse_args()

paths = args.filepaths

xml_generator = pyxmlgeneratorXMLGenerator()

for path in paths:
    path_str = str(path)

    # Validate file
    try:
        xml_generator.validate(path_str)
    except pyxmlgenerator.XSDValidatorError:
        print(f"Invalid file: {path}")
        exit(0)

    print(f"Valid file: {path}")
    try:
        output = xml_generator.generate()
    except:
        print("Error running generator")
        exit(0)

    print("Output generated!")

    # Generate sc
    schema =XMLSchema(path_str)

    if schema.is_valid(output):
        print("Output is valid")
    else:
        print("Output does not match input schema")

```

This example uses Python's [xmlschema library][xmlschema] to validate output XML strings.

### Rust

The following is an example of a simplified program which reads a list of filepaths from command-line arguments and generates example XML output strings for each valid XSD.

```rust
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use xmlgenerator:XMLGenerator;

fn main() {
    let generator =XMLGenerator::new();
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        println!("No files provided!");
        println!("Usage: ./example [files] ...");
        return;
    }

    println!("Reading {} input files...", args.len());
    for arg in args {
        let path = PathBuf::from_str(arg.as_str()).unwrap();

        // Validate file
        if let Err(error) = generator.validate(&path) {
            println!("Invalid file: {}", path);
            return;
        }

        println!("Valid input file: {}", path);
        let output = match generator.generate(&path, None) {
            Ok(out) => out,
            Err(e) => {
                eprintln!("Error running generator: {}", path);
                eprintln!("Error: {}", e);
                return;
            }
        };

        println!("Output generated!");
        println!("Output:");
        println!(output);
    }
}
```

**NOTE:** _Only a single XMLGenerator` should be created. Multiple instances of this class cause undefined behaviour._

## Limitations

Not all features of the XSD specification have been implemented, if these features are encountered, an `unimplemented` error is thrown.

<!-- Links -->

[rust]: https://www.rust-lang.org
[python]: https://www.python.org
[repo]: https://github.com/zwill22/xmlgenerator
[repo-actions]: https://github.com/zwill22/xmlgenerator/actions
[rust-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-rust.yml
[python-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-python.yml
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[xmlschema]: https://pypi.org/project/xmlschema/
[uv]: https://github.com/astral-sh/uv
[website]: https://zmwill.uk
[pytest]: https://docs.pytest.org/en/stable/index.html
[xml]: https://www.w3.org/TR/xml/
[codecov]: https://about.codecov.io/
[coverage]: https://app.codecov.io/gh/zwill22/xmlgenerator
[rtd]: https://about.readthedocs.com/
[doc]: https://xmlgenerator.readthedocs.io/en/latest
[pyo3]: https://pyo3.rs/v0.29.0/
[xsdtests]: https://github.com/w3c/xsdtests
[libxml2-rs]: https://crates.io/crates/libxml2-rs
[libxml2]: https://gitlab.gnome.org/GNOME/libxml2
[crates-repo]: https://crates.io/xml-generator
[github-release]: https://github.com/zwill22/xmlgenerator/releases/latest
[pypi-link]: https://pypi.org/project/pyxmlgenerator/

<!-- Badges -->

[python-badge]: https://img.shields.io/badge/Python-3776AB?logo=python&logoColor=fff&style=for-the-badge
[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[github-badge]: https://img.shields.io/badge/GitHub-%23121011.svg?logo=github&logoColor=white&style=for-the-badge
[actions-badge]: https://img.shields.io/badge/GitHub_Actions-2088FF?logo=github-actions&logoColor=white&style=for-the-badge
[rust-test-badge]: https://img.shields.io/github/actions/workflow/status/zwill22/xmlgenerator/test-rust.yml?style=for-the-badge&logo=rust&label=Test%20Rust
[python-test-badge]: https://img.shields.io/github/actions/workflow/status/zwill22/xmlgenerator/test-python.yml?style=for-the-badge&logo=python&logoColor=white&label=Test%20Python
[coverage-upload-badge]: https://img.shields.io/github/actions/workflow/status/zwill22/xmlgenerator/coverage.yml?style=for-the-badge&logo=codecov&logoColor=white&label=Coverage%20Upload
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[pytest-badge]: https://img.shields.io/badge/pytest-%23ffffff.svg?style=for-the-badge&logo=pytest&logoColor=2f9fe3
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
[uv-badge]: https://img.shields.io/badge/uv-%23DE5FE9.svg?style=for-the-badge&logo=uv&logoColor=white
[codecov-badge]: https://img.shields.io/badge/Codecov-F01F7A?logo=codecov&logoColor=fff&style=for-the-badge
[coverage-badge]: https://img.shields.io/codecov/c/github/zwill22/xmlgenerator?style=for-the-badge&logo=codecov
[rtd-badge]: https://img.shields.io/badge/Read%20the%20Docs-8CA1AF?logo=readthedocs&logoColor=fff&labelColor=333&style=for-the-badge
[doc-badge]: https://img.shields.io/readthedocs/xmlgenerator?style=for-the-badge
[crates-badge]: https://img.shields.io/crates/v/xmlgenerator?style=for-the-badge&logo=rust
[github-release-badge]: https://img.shields.io/github/v/release/zwill22/xmlgenerator?display_name=tag&style=for-the-badge&logo=github
[pypi-version-badge]: https://img.shields.io/pypi/v/pyxmlgenerator?style=for-the-badge&logo=pypi
