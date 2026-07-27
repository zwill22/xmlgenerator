# XSD Test Data

[![Rust][rust-badge]][rust]
[![XML][xml-badge]][xml]
[![License: MIT][license-badge]][license]

This crate packages the [W3C XML Schema 1.1 test suite][test-suite] into a Rust struct `XsdTestData`.
The constructor for which looks for the test suite in the provided file paths.
If these files, do not already exist, then a zip file containing the test repo is downloaded.

The crate provides an iterator for the struct to iterate through the test cases.

## Usage

The following code will download the test data and print a summary of the contents:

```rust
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use xmlgenerator:XmlGenerator;

fn main() {
    let db = PathBuf::from_str("xsdtests-master").unwrap();
    let archive = PathBuf::from_str("xsdtests.zip").unwrap();

    let test_data = XsdTestData::new(&db, &archive);

    test_data.print_stats();
}
```

<!-- Links -->

[rust]: https://www.rust-lang.org
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[xml]: https://www.w3.org/TR/xml/
[test-suite]: https://github.com/w3c/xsdtests

<!-- Badges -->

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
