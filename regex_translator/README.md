# Regex Translator

[![Rust][rust-badge]][rust]
[![XML][xml-badge]][xml]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This crate translates non-standard `xsd:pattern` syntax to a regular expression (regex) pattern compatible with Rust's [regex crate][regex-rust].

The XSD specification provides a pattern syntax for specifying the format of string types.
This syntax is a form of regular expression (regex), however, it differs from the Rust regex syntax in several ways.
In order to make the input patterns rust-compatible, they must be translated.

## Main differences

Some of the key differences which this crate fixes:

- `\i` - shorthand character class denoting any character that can be the first character of an XML name ([full definition here][xml-regex])
- `\c` - shorthand character class denoting any character that can occur after the first character in an XML name ([full definition here][xml-regex])
- `\I` - negated set of `\i`
- `\C` - negated set of `\c`
- Unicode categories `\p{Ll}` and their negative `\P{Ll}` not fully supported by Rust
- Unicode mappings `\p{IsCyrillic}` and the negative `\P{IsCyrillic}` not fully supported by Rust
- Or operator `[a|b]` in XSD not supported by Rust, require `[ab]`
- Inconsistent usage of `\w`, `\W`, `\d`, `\D`, and `.` between the two

In addition, the crate standardises unicode characters references and checks line endings.

## Validation

The input patterns are validated using the [regexml][regexml] crate.
Output patterns are validated using the [regex][regex-rust] crate.

<!-- Links -->

[rust]: https://www.rust-lang.org
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[website]: https://zmwill.uk
[xml]: https://www.w3.org/TR/xml/
[regex-rust]: https://docs.rs/regex/latest/regex/
[regexml]: https://docs.rs/regexml/latest/regexml/
[xml-regex]: https://www.regular-expressions.info/shorthand.html

<!-- Badges -->

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&style=for-the-badge
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
