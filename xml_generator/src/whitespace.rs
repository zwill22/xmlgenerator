use line_ending::LineEnding;
use regex::Regex;
use crate::XMLGeneratorError;

fn check_line_ending(pattern: &str, ending: &str) -> Result<(), XMLGeneratorError> {
    if pattern.contains(ending) {
        return Err(XMLGeneratorError::LineEndingsError(pattern.to_string()));
    }

    Ok(())
}

fn check_crlf_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    let stripped = pattern.replace("\\r\\n", "");

    if stripped.contains("\\r") || stripped.contains("\\n") {
        return Err(XMLGeneratorError::LineEndingsError(pattern.to_string()));
    }

    Ok(())
}

fn check_line_endings(pattern: &str) -> Result<(), XMLGeneratorError> {
    match LineEnding::from_current_platform() {
        LineEnding::LF => check_line_ending(pattern, r"\r"),
        LineEnding::CRLF => check_crlf_endings(pattern),
        LineEnding::CR => check_line_ending(pattern, r"\n"),
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
    let whitespace = Regex::new(r"\[\^(?:\\s|\\t|\\n|\\r|\\v|\\f)+]").unwrap();

    whitespace.replace_all(input, r"[\S ]").to_string()
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
            },
            WhiteSpace::Replace => replace(s),
            WhiteSpace::Collapse => collapse(s),
        };

        Ok(output)
    }
}
