import os
import pytest
import subprocess

from pathlib import Path
from .common import get_project_root
from pyxmlgenerator import XMLGenerator

def build_and_run_lib():
    stdout = subprocess.PIPE
    stderr = subprocess.PIPE

    p = subprocess.run(["cargo", "run"], stdout=stdout, stderr=stderr)

    try:
        p.check_returncode()
    except subprocess.CalledProcessError as e:
        raise RuntimeError(e)


def fetch_data():
    build_and_run_lib()


def fetch_xsd_test_data(workspace_root: Path, start_dir: Path):
    xsd_test_data_dir = workspace_root / "xsd-test-data"
    os.chdir(xsd_test_data_dir)
    fetch_data()
    os.chdir(start_dir)
    pass


@pytest.fixture(scope="session")
def xml_generator() -> XMLGenerator:
    return XMLGenerator()


directory = Path.cwd()
root = get_project_root()

test_data_dir = root / 'xsdtests-master'
if not test_data_dir.exists():
    fetch_xsd_test_data(root, directory)
