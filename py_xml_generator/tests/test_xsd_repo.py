import pytest
from pathlib import Path

import pyxmlgenerator
from pyxsdtestdata import XSDTestData

from .common import validate_output, get_project_root


xsd_directory = "xsdtests-master"
archive_filename = "xsdtests.zip"
extras = True

root = get_project_root()
xsd_filepath = root / xsd_directory
archive_filepath = root / archive_filename

def validate_schema(xml_generator, xsd_file: Path):
    filepath = str(xsd_file)
    xml_generator.validate(filepath)

xsd_test_data = XSDTestData(xsd_filepath, archive_filepath)

def get_valid_files():
    result = []
    for data in xsd_test_data:
        if data.valid():
            result.append(data.key())

    return result

valid_files = get_valid_files()

@pytest.mark.parametrize("file", valid_files)
def test_xsd_file(xml_generator, file):
    data = xsd_test_data.get(file)
    path = data.path()

    try:
        validate_schema(xml_generator, path)
    except pyxmlgenerator.XSDValidatorError:
        pytest.skip(f"Unable to validate XSD: {file}")

    validate_output(xml_generator, path)

# def test_single_file(xml_generator):
#     file = "msData/regex/reI57.xsd"
#     filepath = xsd_filepath / file
#     test_xsd_file(xml_generator, filepath)
