# PyXSDTestData

[![Python][python-badge]][python]
[![XML][xml-badge]][xml]
[![uv][uv-badge]][uv]
[![License: MIT][license-badge]][license]
[![No AI][noai-badge]][website]

This package exposes the [xsd-test-data](../xsd_test_data/README.md) crate to Python using the [PyO3 framework][pyo3].
The package provides a Python class `XSDTestData` which directly corresponds to the `XSDTestData` Rust struct.

## Usage

The following code will download the test data and print a summary of the contents:

```python
from pathlib import Path

from pyxsdtestdata import XSDTestData


xsd_directory = Path("xsdtests-master")
archive_filename = Path("xsdtests.zip")

xsd_test_data = XSDTestData(xsd_filepath, archive_filepath)

valid_files = 0
for data in xsd_test_data:
    if data.valid():
        valid_files += 1

print(f"Valid test files = {valid_files}")
```

Similar to the Rust implementation, initialising an `XSDTestData` object will cause the test data to be downloaded if the specified files do not exist.

<!-- Links -->

[python]: https://www.python.org
[license]: https://github.com/zwill22/OpenBusAPI/blob/main/LICENSE
[uv]: https://github.com/astral-sh/uv
[website]: https://zmwill.uk
[xml]: https://www.w3.org/TR/xml/
[pyo3]: https://pyo3.rs/v0.29.0/

<!-- Badges -->

[python-badge]: https://img.shields.io/badge/Python-3776AB?logo=python&logoColor=fff&style=for-the-badge
[license-badge]: https://img.shields.io/github/license/zwill22/xmlgenerator?style=for-the-badge
[noai-badge]: https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&style=for-the-badge
[xml-badge]: https://img.shields.io/badge/XML-767C52?logo=xml&logoColor=fff&style=for-the-badge
[uv-badge]: https://img.shields.io/badge/uv-%23DE5FE9.svg?style=for-the-badge&logo=uv&logoColor=white
