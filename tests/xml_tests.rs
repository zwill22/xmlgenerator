use std::io::Write;
use std::process::{Command, Output, Stdio};

use xsd_parser::Error;
pub fn rustfmt_pretty_print(code: String) -> Result<String, Error> {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();

    write!(stdin, "{code}")?;
    stdin.flush()?;
    drop(stdin);

    let Output {
        status,
        stdout,
        stderr,
    } = child.wait_with_output()?;

    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);

    if !status.success() {
        let code = status.code();
        match code {
            Some(code) => {
                if code != 0 {
                    panic!("The `rustfmt` command failed with return code {code}!\n{stderr}");
                }
            }
            None => {
                panic!("The `rustfmt` command failed!\n{stderr}")
            }
        }
    }

    Ok(stdout.into())
}

#[cfg(test)]
mod tests {
    use std::{fs, path};
    use std::fs::ReadDir;
    use xmlgenerator::generate_xml;
    use crate::rustfmt_pretty_print;

    fn fetch_test_files() -> ReadDir {
        let example_dir = path::absolute("./examples").unwrap();
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    #[test]
    fn test_xml() {
        let files = fetch_test_files();

        for file in files {
            let filepath = file.unwrap().path();
            println!("{}", filepath.display());
            let xml = generate_xml(filepath.into_boxed_path());

            if xml.is_ok() {
                print!("{}", rustfmt_pretty_print(xml.unwrap()).unwrap());
            } else {
                println!("{}", xml.err().unwrap());
            }
        }
    }
}
