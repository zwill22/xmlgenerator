# XSD Validator

[![Rust][rust-badge]][rust]
[![XML][xml-badge]][xml]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This crate uses the [libxml2][libxml2] C library to validate XSD input files.
Additionally, it also checks for infinite loops within the input tree.

## Dependencies

- [libxml2-rs][libxml2-rs] - Rust bindings for the libxml2 C library
- [xsd-test-data](../xsd_test_data/README.md) - For testing

## Example Usage

The following is an example of a simplified program which reads a list of filepaths from command-line arguments and generates example XML output strings for each valid XSD.

```rust
use std::path::PathBuf;
use std::str::FromStr;
use xsd_validator::XsdValidator;

fn main() {
    let validator = XsdValidator::new(true);

    let path = PathBuf::from_str("example.xsd").unwrap();

    match validator.validate(path) {
        Ok(valid) => {
            if valid {
                println!("File is valid!");
            } else {
                println!("File is invalid!");
            }
        }
        Err(e) => {
            eprintln!("Error validating file: {}", e);
        }
    };
}
```

**NOTE:** _Only a single `XsdValidator` should be created at once. Multiple instances of this class cause undefined behaviour._

Note that no error is thrown if the file is invalid, only if an error occurs during validation.

## File validity

This crate uses the [libxml2][libxml2] for validation.
Additional checks relating to infinite loops are performed.
Otherwise, whether a file is valid depends entirely on whether `libxml2` interprets it as valid or not.

<!-- Links -->

[rust]: https://www.rust-lang.org
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[website]: https://zmwill.uk
[xml]: https://www.w3.org/TR/xml/
[libxml2]: https://gitlab.gnome.org/GNOME/libxml2
[libxml2-rs]: https://github.com/zwill22/libxml2-rs

<!-- Badges -->

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
