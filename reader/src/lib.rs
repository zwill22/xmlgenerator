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
        if index >= self.list.len() {
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
    let file = File::open(file_path).expect("failed to open file");
    let encoding = match encodings.get(index) {
        Some(encoding) => encoding,
        None => return Err(Error::new(ErrorKind::InvalidData, "cannot read file")),
    };

    let mut reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(file),
    );

    let mut string = String::new();
    match reader.read_to_string(&mut string) {
        Ok(_) => Ok(string),
        Err(_) => read_file_encoding(&file_path, &encodings, index + 1),
    }
}

pub fn read_file(filepath: &PathBuf) -> Result<String, Error> {
    let encodings = Encodings::generate();

    match read_to_string(&filepath) {
        Ok(s) => Ok(s),
        Err(_) => read_file_encoding(&filepath, &encodings, 0),
    }
}
