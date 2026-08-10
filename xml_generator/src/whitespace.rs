use crate::{error::unimplemented, XMLGeneratorError};
use line_ending::LineEnding;
use regex::Regex;

fn check_lf_line_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    if pattern.contains(r"\r") {
        return Err(XMLGeneratorError::LineEndingsError(pattern.to_string()));
    }

    Ok(())
}

fn check_crlf_line_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    let stripped = pattern.replace(r"\r\n", r"");

    if stripped != pattern {
        return unimplemented("CRLF line endings");
    }

    if stripped.contains(r"\r") {
        return Err(XMLGeneratorError::LineEndingsError(pattern.to_string()));
    }

    Ok(())
}

fn check_cr_line_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    if pattern.contains(r"\n") {
        return Err(XMLGeneratorError::LineEndingsError(pattern.to_string()));
    }

    match pattern.find(r"\r") {
        Some(_) => unimplemented("CR line endings"),
        None => Ok(()),
    }
}

pub(crate) fn check_line_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    match LineEnding::from_current_platform() {
        LineEnding::LF => check_lf_line_endings(pattern),
        LineEnding::CRLF => check_crlf_line_endings(pattern),
        LineEnding::CR => check_cr_line_endings(pattern),
    }
}

#[derive(PartialEq, Default, Clone)]
pub(crate) enum WhiteSpace {
    #[default]
    Preserve,
    Replace,
    Collapse,
}

fn replace(input: &str) -> String {
    let whitespace = Regex::new(r"\[\^(?:\\t|\\n|\\r|\\v|\\f)+]").unwrap();

    whitespace
        .replace_all(input, r"[\S ]")
        .replace(r"[^\s]", r"\S")
        .replace(r"\s", " ")
        .replace(r"\t", "")
        .replace(r"\n", " ")
        .replace(r"\r", " ")
        .replace(r"\v", " ")
        .replace(r"\f", " ")
}

fn collapse(input: &str) -> String {
    replace(input).replace(" ", "")
}

impl WhiteSpace {
    pub(crate) fn handle(&self, s: &str) -> Result<String, XMLGeneratorError> {
        let output = match self {
            WhiteSpace::Preserve => {
                check_line_endings(s)?;
                s.to_string()
            }
            WhiteSpace::Replace => replace(s),
            WhiteSpace::Collapse => collapse(s),
        };

        Ok(output)
    }
}
