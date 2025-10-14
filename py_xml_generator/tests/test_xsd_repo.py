import pytest
from pathlib import Path
from typing import List

import pyxmlgenerator

from .common import validate_output, get_project_root


def list_from_file(xsd_list: Path, xsd_dir: Path) -> List[Path]:
    with open(xsd_list, 'r') as f:
        files = f.readlines()

    return [xsd_dir / file for file in files]


def list_from_dir(xsd_dir: Path) -> List[Path]:
    files = xsd_dir.rglob("*.xsd")
    output = []
    for xsd_file in files:
        output.append(xsd_file)
    return output


def get_file_list(root_dir: Path, xsd_dir: Path) -> List[Path]:
    xsd_list = root_dir / "xsd_list.txt"
    if xsd_list.is_file():
        return list_from_file(xsd_list, xsd_dir)
    else:
        return list_from_dir(xsd_dir)


def fetch_xsd_files() -> List[Path]:
    root_dir = get_project_root()
    xsd_dir = root_dir / "xsdtests-master"

    assert xsd_dir.is_dir()

    files = get_file_list(root_dir, xsd_dir)

    return files


data = fetch_xsd_files()


def id_fn(file: Path):
    return str(file)


def validate_schema(xml_generator, xsd_file: Path):
    filepath = str(xsd_file)
    xml_generator.validate(filepath)


@pytest.mark.parametrize("file", data, ids=id_fn)
def test_xsd_file(xml_generator, file):
    try:
        validate_schema(xml_generator, file)
    except pyxmlgenerator.XSDValidatorError:
        return

    try:
        validate_output(xml_generator, file)
    except pyxmlgenerator.NoElementsError:
        return
    except pyxmlgenerator.ImplementationError as e:
        print(e)
        return
