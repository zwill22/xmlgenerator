use crate::XMLGeneratorError;
use crate::whitespace::WhiteSpace;
use html_escape::encode_text;

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
