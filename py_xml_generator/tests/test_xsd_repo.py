from typing import List

import pytest
from pathlib import Path

import pyxmlgenerator
from pyxsdtestdata import get_xsd_test_data

from .common import validate_output, get_project_root, identity


def fetch_xsd_files(xsd_root_path: Path, archive_path: Path, include_extras: bool) -> List[Path]:
    data = get_xsd_test_data(xsd_root_path, archive_path, include_extras)

    valid_files = []
    for entry in data:
        assert type(entry) is tuple
        assert len(entry) == 2

        path: Path = entry[0]
        valid: bool = entry[1]

        if valid:
            valid_files.append(path)

    return valid_files


xsd_directory = "xsdtests-master"
archive_filename = "xsdtests.zip"
extras = True

root = get_project_root()
xsd_filepath = root / xsd_directory
archive_filepath = root / archive_filename

valid_data = fetch_xsd_files(xsd_filepath, archive_filepath, extras)


def validate_schema(xml_generator, xsd_file: Path):
    filepath = str(xsd_file)
    xml_generator.validate(filepath)


def id_fn(file: Path) -> str:
    return identity(file, xsd_filepath)


@pytest.mark.parametrize("file", valid_data, ids=id_fn)
def test_xsd_file(xml_generator, file):
    try:
        validate_schema(xml_generator, file)
    except pyxmlgenerator.XSDValidatorError:
        pytest.skip("Unable to validate XSD")

    validate_output(xml_generator, file)

# def test_single_file(xml_generator):
#     file = "msData/regex/reI57.xsd"
#     filepath = xsd_filepath / file
#     test_xsd_file(xml_generator, filepath)
