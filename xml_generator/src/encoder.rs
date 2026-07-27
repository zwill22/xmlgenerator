use crate::XMLGeneratorError;
use crate::whitespace::WhiteSpace;
use html_escape::encode_text;
use unic_char_basics::{is_noncharacter, is_private_use};
use unic_ucd::CharAge;

pub(crate) fn decode_html(input: &str) -> String {
    html_escape::decode_html_entities(input).to_string()
}

fn filter(input: &str, whitespace: &WhiteSpace) -> Result<String, XMLGeneratorError> {
    let mut out = "".to_string();
    for c in input.chars() {
        if c.is_whitespace() {
            match whitespace {
                WhiteSpace::Preserve => out.push(c),
                WhiteSpace::Replace => out.push(' '),
                WhiteSpace::Collapse => {}
            }
        } else if c.is_control() {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "Control characters".to_string(),
            ));
        } else if is_noncharacter(c) {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "Non characters".to_string(),
            ));
        } else if is_private_use(c) {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "Private use characters".to_string(),
            ));
        } else if c.age().is_none() {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "Unassigned Unicode characters".to_string(),
            ));
        } else {
            out.push(c);
        }
    }

    Ok(out)
}

pub(crate) fn encode_html(
    input: &str,
    whitespace: &WhiteSpace,
) -> Result<String, XMLGeneratorError> {
    let tmp = decode_html(input);

    let filtered = filter(&tmp, whitespace)?;

    let output = encode_text(&filtered).to_string();

    Ok(output)
}
