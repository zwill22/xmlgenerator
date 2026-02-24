use regex::Regex;

pub(crate) fn replace_specials(input: &str) -> String {
    let regex = Regex::new(r"&#x(\w+);").unwrap();

    let out1 = regex
        .replace_all(input, r"\x{$1}")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot", "\"")
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;");

    let regex2 = Regex::new(r"\\x\{(\w+)}").unwrap();

    regex2.replace_all(&out1, r"&#x$1;").to_string()
}
