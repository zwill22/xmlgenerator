import pytest

from pathlib import Path

from .common import validate_output, get_project_root, identity

project_root: Path = get_project_root()
example_dir: Path = project_root / "examples" / "working"


def get_files() -> list[Path]:
    out = []
    for file in example_dir.iterdir():
        out.append(file)

    return out


def id_fn(file: Path) -> str:
    return identity(file, example_dir)


@pytest.mark.parametrize("input_file", get_files(), ids=id_fn)
def test_files(xml_generator, input_file):
    validate_output(xml_generator, input_file)
