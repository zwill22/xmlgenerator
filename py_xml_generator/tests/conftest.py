import pytest

from pathlib import Path
from .common import get_project_root
from pyxmlgenerator import XMLGenerator
from pyxsdtestdata import get_xsd_test_data

@pytest.fixture(scope="session")
def xml_generator() -> XMLGenerator:
    return XMLGenerator()


directory = Path.cwd()
root = get_project_root()

test_data_dir = root / 'xsdtests-master'
if not test_data_dir.exists():
    test_data_archive = root / "xsdtests.zip"

    get_xsd_test_data(test_data_dir, test_data_archive, True)
