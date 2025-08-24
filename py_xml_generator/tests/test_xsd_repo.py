import pytest
from git import Repo
from pathlib import Path
from typing import List

from xmlschema import XMLSchemaValidationError, XMLSchemaParseError, XMLSchemaModelError
from xmlschema.exceptions import XMLResourceParseError

from .common import validate_output, get_project_root


def fetch_xsd_data() -> List[Path]:
    root_dir = get_project_root()
    local_dir = root_dir / "xsdtests-master"

    if local_dir.is_dir():
        xsd_dir = local_dir
    else:
        remote: str = "https://github.com/w3c/xsdtests.git"
        repo = Repo.clone_from(remote, local_dir)
        xsd_dir = repo.working_dir

    files = Path(xsd_dir).rglob("*.xsd")

    output = []
    for file in files:
        output.append(file)

    return output

data = fetch_xsd_data()

def idfn(file: Path):
    return str(file)

@pytest.mark.parametrize("file", data, ids=idfn)
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
