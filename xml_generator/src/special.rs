use crate::whitespace::WhiteSpace;

pub(crate) fn replace_specials(input: &str, whitepace: &WhiteSpace) -> String {
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
                WhiteSpace::Collapse => {},
            }
        } else if c.is_control() {
            let code = c as u32;

            let char_str = format!("&#x{:04x};", code);
            output.push_str(&char_str);
        } else {
            output.push(c);
        }
    }

    output
}
