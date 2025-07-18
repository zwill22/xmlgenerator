import pytest
import xmlschema

from pathlib import Path
from typing import List
from xmlgenerator import generate_xml_from_xsd
from xmlgenerator.xsd2xml import CircularReferenceError

__root_dir__ = Path(__file__).parent.parent


def get_xsd_files() -> List[Path]:
    examples_path = __root_dir__ / "examples"
    output = [path for path in examples_path.iterdir()]
    return output


def test_empty_path():
    get_xsd_files()
    with pytest.raises(FileNotFoundError):
        generate_xml_from_xsd("")


def validate(xml: str, xsd_file: Path | str) -> None:
    schema = xmlschema.XMLSchema(xsd_file)

    try:
        schema.validate(xml)
    except xmlschema.XMLSchemaException as e:
        pytest.fail(e)


@pytest.mark.parametrize("xsd_file", get_xsd_files())
def test_xsd(xsd_file):
    try:
        xml = generate_xml_from_xsd(str(xsd_file))
        assert xml is not None
        validate(xml, xsd_file)
    except CircularReferenceError as e:
        assert e is not None
