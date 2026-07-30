//! Rust crate for reading text files into strings when the enccoding is unknown
//! 
//! This crate provides a single function [read_file] which attempts to read a text file.
//! If it fails to read the file using the [std::fs::read_to_string] function, 
//! it retries using various standard encodings until it finds one that works. 

use encoding_rs::{Encoding, UTF_16LE};
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::fs::File;
use std::fs::read_to_string;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::path::PathBuf;

struct Encodings {
    list: Vec<&'static Encoding>,
}

impl Encodings {
    fn generate() -> Encodings {
        let mut encodings = Encodings { list: Vec::new() };

        encodings.list.push(UTF_16LE);

        encodings
    }

    fn get(&self, index: usize) -> Option<&'static Encoding> {
        if (index as u32) >= (self.list.len() as u32) {
            return None;
        }

        Some(self.list[index])
    }
}

fn read_file_encoding(
    file_path: &PathBuf,
    encodings: &Encodings,
    index: usize,
) -> Result<String, Error> {
    let file = File::open(file_path)?;
    let encoding = match encodings.get(index) {
        Some(encoding) => encoding,
        None => {
            return Err(Error::new(ErrorKind::InvalidData, "cannot read file"));
        }
    };

    let mut reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(file),
    );

    let mut string = String::new();
    match reader.read_to_string(&mut string) {
        Ok(_) => Ok(string),
        Err(_) => read_file_encoding(file_path, encodings, index + 1),
    }
}

/// Reads a file to a string for a file with unknown encoding
/// 
/// # Arguments
/// 
/// - `filepath` (`&PathBuf`) - Path to the file
/// 
/// # Returns
/// 
/// - `Result<String, Error>` - Returns the file string if no error is found
/// 
/// # Errors
/// 
/// - [std::io::Error] - If file cannot be decoded using any of the listed encodings
/// 
/// # Example
/// 
/// ```rust
/// use file_to_string::read_file;
/// 
/// fn read(file: &PathBuf) -> String {
///     match read_file(file) {
///         Ok(out) => {
///             return out;
///         },
///         Err(e) => {
///             eprintln!("Error reading file: {}", e);
///             return "".to_string();
///         }
///     }
/// }
/// ```
/// 
pub fn read_file(filepath: &PathBuf) -> Result<String, Error> {
    let encodings = Encodings::generate();

    match read_to_string(filepath) {
        Ok(s) => Ok(s),
        Err(_) => read_file_encoding(filepath, &encodings, 0),
    }
}
