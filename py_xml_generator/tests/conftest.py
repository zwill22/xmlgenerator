import pytest
import os

if os.name == "nt":
    for path in os.environ["PATH"].split(";"):
        try:
            os.add_dll_directory(str(path))
        except FileNotFoundError:
            pass


from pathlib import Path
from .common import get_project_root
from pyxmlgenerator import XMLGenerator


@pytest.fixture(scope="session")
def xml_generator() -> XMLGenerator:
    return XMLGenerator()


directory = Path.cwd()
root = get_project_root()

test_data_dir = root / "xsdtests-master"
test_data_archive = root / "xsdtests.zip"
