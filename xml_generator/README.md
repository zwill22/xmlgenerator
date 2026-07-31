# XML Generator

[![Rust][rust-badge]][rust]
[![XML][xml-badge]][xml]
[![GitHub][github-badge]][repo]
[![Pytest][pytest-badge]][pytest]
[![GitHub Actions][actions-badge]][repo-actions]
[![Test Rust][rust-test-badge]][rust-test]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This crate is the primary Rust crate in the repo.
It uses the [xsd-parser][xsd-parser] crate to parse the input XSD into a `SchemaData` object.
Next, the XSD data is validated using the [xsd-validator](../xsd_validator/README.md).
A `Generator` object is created for managing the generation of randomised strings.
These objects are then combined into an `Xsd` object, this object sorts the schema content into a new structure primed for generating a new XML instance.
After this, the new XML is generated.

## Randomised Generation

The crate uses the [rand-regex][rand-regex] crate to generate regex strings based on the input pattern.
This process is random, and therefore non-deterministic.
For more complicated regex patterns, this can fail, but repeated attempts may yield a suitable output.

## Example Usage

The following is an example of a simplified program which reads a list of filepaths from command-line arguments and generates example XML output strings for each valid XSD.

```rust
use std::path::PathBuf;
use std::str::FromStr;
use xmlgenerator:XmlGenerator;

fn main() {
    let generator =XmlGenerator::new();

    let path = PathBuf::from_str("input.xsd").unwrap();

    // Unseeded
    let output = match generator.validate(&path, None) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error generating XML: {}", e);
            return;
        }
    };

    println!("Output XML generated!");

    println!("{}", output);
}
```

**NOTE:** _Only a single XmlGenerator` should be created. Multiple instances of this class cause undefined behaviour._

<!-- Links -->

[rust]: https://www.rust-lang.org
[repo]: https://github.com/zwill22/xmlgenerator
[repo-actions]: https://github.com/zwill22/xmlgenerator/actions
[rust-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-rust.yml
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[website]: https://zmwill.uk
[pytest]: https://docs.pytest.org/en/stable/index.html
[xml]: https://www.w3.org/TR/xml/
[xsd-parser]: https://docs.rs/xsd-parser/latest/xsd_parser/
[rand-regex]: https://docs.rs/rand_regex/latest/rand_regex/

<!-- Badges -->

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[github-badge]: https://img.shields.io/badge/GitHub-%23121011.svg?logo=github&logoColor=white&style=for-the-badge
[actions-badge]: https://img.shields.io/badge/GitHub_Actions-2088FF?logo=github-actions&logoColor=white&style=for-the-badge
[rust-test-badge]: https://img.shields.io/github/actions/workflow/status/zwill22/xmlgenerator/test-rust.yml?style=for-the-badge&logo=rust&label=Test%20Rust
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[pytest-badge]: https://img.shields.io/badge/pytest-%23ffffff.svg?style=for-the-badge&logo=pytest&logoColor=2f9fe3
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
