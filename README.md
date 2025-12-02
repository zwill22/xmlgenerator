# XMLGenerator

[![Rust][rust-badge]][rust]
[![Python][python-badge]][python]
[![GitHub][github-badge]][repo]
[![GitHub Actions][actions-badge]][repo-actions]
[![Test Rust][rust-test-badge]][rust-test]
[![Test Python][python-test-badge]][python-test]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]


This project is a Rust library which generates fake XML output for the provided XSD input.
Also provided is a Python wrapper `py-xml-generator` allowing direct use from Python.
The project includes several Cargo crates to achieve this goal. 

## Dependencies

- [libxml2-rs][libxml2-rs] - Custom Rust bindings for the [libxml2][libxml2] C library
- [XSD-parser](https://github.com/Bergmann89/xsd-parser) - Parsing input XSD schemas
- [xml-builder](https://github.com/cocool97/xml-builder) - Building output XML
- [fake-rs](https://github.com/cksac/fake-rs) - Generating fake data
- [regexml](https://github.com/Paligo/regexml) - XML Regex validation
- [rand_regex](https://github.com/kennytm/rand_regex) - Generating random string from Regex
- [roxmltree](https://github.com/RazrFalcon/roxmltree) - Parsing XML
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP Client
- [PyO3](https://github.com/PyO3/pyo3) - Python Bindings
- [Polars](https://pola.rs) - Reading tabular data

## Usage (Rust)

The following is an example of a simplified program which reads a list of filepaths from command-line arguments and generates example XML output strings for each valid XSD.

```rust
use std::path::PathBuf;
use xmlgenerator::XMLGenerator;
use std::env;

fn main() {
    let generator = XMLGenerator::new();
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
        let output = match generator.generate(&path) {
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

**NOTE:** *Only a single `XMLGenerator` should be created. Multiple instances of this class cause undefined behaviour.*

# Python Example

The following example illustrates a similar workflow in Python, using the built-in [argparse](https://docs.python.org/3/library/argparse.html) and [pathlib](https://docs.python.org/3/library/pathlib.html) librarys to read and validate command-line input.
```python
import pathlib
import argparse
import pyxmlgenerator

parser = argparse.ArgumentParser()
parser.add_argument("filepaths", type=pathlib.Path, nargs='+')

args = parser.parse_args()

paths = args.filepaths

xml_generator = pyxmlgenerator.XMLGenerator()

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
    print(output)
```

[//]: # (Links)
[rust]: https://www.rust-lang.org
[python]: https://www.python.org
[repo]: https://github.com/zwill22/xmlgenerator
[repo-actions]: https://github.com/zwill22/xmlgenerator/actions
[rust-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-rust.yml
[python-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-python.yml
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[libxml2]: https://gitlab.gnome.org/GNOME/libxml2
[libxml2-rs]: https://github.com/zwill22/libxml2-rs.git

[//]: # (Badges)
[python-badge]: https://img.shields.io/badge/Python-3776AB?logo=python&logoColor=fff
[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white
[github-badge]: https://img.shields.io/badge/GitHub-%23121011.svg?logo=github&logoColor=white
[actions-badge]: https://img.shields.io/badge/GitHub_Actions-2088FF?logo=github-actions&logoColor=white
[rust-test-badge]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-rust.yml/badge.svg
[python-test-badge]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-python.yml/badge.svg
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator
