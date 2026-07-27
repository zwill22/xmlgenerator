# File-To-String

[![Rust][rust-badge]][rust]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This crate provides a single function `read_file` to read text files with unknown encoding.
The function attempts to read the file with the standard `file_to_string` function.
If this fails, it generates a list of standard file encodings and attempts to read the input file using each one.
If the file does not match an encoding, an error is thrown and the next encoding in the list is tried.

## Usage

To use the `read_file` function

```rust
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use file_to_string::read_file;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        println!("No files provided!");
        println!("Usage: ./example [file]");
        return;
    } else if args.len() > 1 {
        println!("Multiple files provided!");
        println!("Please provide a single text file");
    }

    let file = &args[0];
    println!("Reading file: {}", file);

    let path: PathBuf = PathBuf::from_str(file.as_str()).unwrap();

    let file_string = match read_file(&path) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Unable to read file: {:?}", path);
            eprintln!("Error: {}", e);
            return;
        }
    };

    println!("Successfully read file");
    println!("Contents:");
    println!("{}", file_string);
}
```

<!-- Links -->

[rust]: https://www.rust-lang.org
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[website]: https://zmwill.uk

<!-- Badges -->

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
