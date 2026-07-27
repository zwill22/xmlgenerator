# PyXMLGenerator

[![Python][python-badge]][python]
[![XML][xml-badge]][xml]
[![GitHub][github-badge]][repo]
[![uv][uv-badge]][uv]
[![Pytest][pytest-badge]][pytest]
[![GitHub Actions][actions-badge]][repo-actions]
[![Test Python][python-test-badge]][python-test]
[![Coffee][buy-me-coffee]][coffee]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This package is a Python wrapper for the `xml-generator` library, created using the [PyO3 framework][pyo3].
The package exposes an `XMLGenerator` class, which corresponds to the Rust struct of the same name.

## Example usage

The following example generates an XML instance from the provided XSD file and uses the `xmlschema` package to validate the output:

```python
from xmlschema import XMLSchema
from pyxmlgenerator import XMLGenerator, XMLValidatorError

xml_generator = XMLGenerator()

path = "input.xsd"

try:
    xml_generator.validate(path_str)
except pyxmlgenerator.XSDValidatorError:
    print(f"Invalid file: {path}")
    exit(0)

print(f"Valid file: {path})


output = xml_generator.generate()

print("Output generated!")

# Generate schema
schema = XMLSchema(path)

# Check output against schema
if schema.is_valid(output):
    print("Output is valid")
else:
    print("Output does not match input schema")

```

This example uses Python's [xmlschema library][xmlschema] to validate output XML strings.

<!-- Links -->

[python]: https://www.python.org
[repo]: https://github.com/zwill22/xmlgenerator
[repo-actions]: https://github.com/zwill22/xmlgenerator/actions
[python-test]: https://github.com/zwill22/xmlgenerator/actions/workflows/test-python.yml
[coffee]: https://coff.ee/zmwill
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[xmlschema]: https://pypi.org/project/xmlschema/
[uv]: https://github.com/astral-sh/uv
[website]: https://zmwill.uk
[pytest]: https://docs.pytest.org/en/stable/index.html
[xml]: https://www.w3.org/TR/xml/
[pyo3]: https://pyo3.rs/v0.29.0/

<!-- Badges -->

[python-badge]: https://img.shields.io/badge/Python-3776AB?logo=python&logoColor=fff&style=for-the-badge
[github-badge]: https://img.shields.io/badge/GitHub-%23121011.svg?logo=github&logoColor=white&style=for-the-badge
[actions-badge]: https://img.shields.io/badge/GitHub_Actions-2088FF?logo=github-actions&logoColor=white&style=for-the-badge
[python-test-badge]: https://img.shields.io/github/actions/workflow/status/zwill22/xmlgenerator/test-python.yml?style=for-the-badge&logo=python&logoColor=white&label=Test%20Python
[buy-me-coffee]: https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?logo=buy-me-a-coffee&logoColor=black&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[pytest-badge]: https://img.shields.io/badge/pytest-%23ffffff.svg?style=for-the-badge&logo=pytest&logoColor=2f9fe3
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
[uv-badge]: https://img.shields.io/badge/uv-%23DE5FE9.svg?style=for-the-badge&logo=uv&logoColor=white
