import pytest
from git import Repo
from pathlib import Path
from typing import List

from xmlschema import XMLSchemaValidationError, XMLSchemaParseError, XMLSchemaModelError
from xmlschema.exceptions import XMLResourceParseError

from .common import validate_output, get_project_root


def list_from_file(xsd_list: Path, xsd_dir: Path) -> List[Path]:
    with open(xsd_list, 'r') as f:
        files = f.readlines()

    return [ xsd_dir / file for file in files ]

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

def output_list(output: List[Path], root_dir: Path, xsd_dir: Path):
    output_file = root_dir / "xsd_list.txt"

    if output_file.is_file():
        return

    with open(output_file, 'w') as f:
        for file in output:
            f.write(f"{str(file.relative_to(xsd_dir))}\n")


def fetch_xsd_data() -> List[Path]:
    root_dir = get_project_root()
    local_dir = root_dir / "xsdtests-master"

    if local_dir.is_dir():
        xsd_dir = local_dir
    else:
        remote: str = "https://github.com/w3c/xsdtests.git"
        repo = Repo.clone_from(remote, local_dir)
        xsd_dir = Path(repo.working_dir)

    files = get_file_list(root_dir, xsd_dir)

    output_list(files, root_dir, xsd_dir)

    return []

data = fetch_xsd_data()

def id_fn(file: Path):
    return str(file)

@pytest.mark.parametrize("file", data, ids=id_fn)
def test_xsd_file(file):
    try:
        validate_output(file)
    except RuntimeError as e:
        for arg in e.args:
            errors = [
                "XSD Parser encountered an error",
                "Implementation error"
            ]

            for e in errors:
                if e in arg:
                    return

        raise e
