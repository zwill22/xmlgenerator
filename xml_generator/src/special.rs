use crate::XMLGeneratorError;
use crate::whitespace::WhiteSpace;

pub(crate) fn replace_specials(
    input: &str,
    whitepace: &WhiteSpace,
) -> Result<String, XMLGeneratorError> {
    let out1 = input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot", "\"");

    let mut output = "".to_string();
    for c in out1.chars() {
        if c.eq(&'&') {
            output.push_str("&amp;");
        } else if c.eq(&'<') {
            output.push_str("&lt;");
        } else if c.eq(&'>') {
            output.push_str("&gt;");
        } else if c.eq(&'"') {
            output.push_str("&quot;");
        } else if c.is_whitespace() {
            match whitepace {
                WhiteSpace::Preserve => output.push(c),
                WhiteSpace::Replace => output.push(' '),
                WhiteSpace::Collapse => {}
            }
        } else if c.is_control() {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "Control characters".to_string(),
            ));
        } else {
            output.push(c);
        }
    }

    Ok(output)
}
