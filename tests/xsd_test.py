import pytest

from pathlib import Path
from typing import List
from xmlgenerator import generate_xml_from_xsd

__root_dir__ = Path(__file__).parent.parent

from xmlgenerator.xsd2xml import CircularReferenceError


def get_xsd_files() -> List[Path]:
    examples_path = __root_dir__ / "examples"
    output = [path for path in examples_path.iterdir()]
    return output

def test_empty_path():
    get_xsd_files()
    with pytest.raises(FileNotFoundError):
        generate_xml_from_xsd("")

def test_xsd():
    xsd_files = get_xsd_files()
    for xsd_file in xsd_files:
        try:
            xml = generate_xml_from_xsd(str(xsd_file))
            assert xml is not None
        except CircularReferenceError as e:
            assert e is not None
