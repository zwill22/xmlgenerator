import pytest

from pathlib import Path
from common import validate_output, get_project_root


def get_files() -> list[Path]:
    project_root: Path = get_project_root()
    example_dir: Path = project_root / "examples" / "working"

    return [file for file in example_dir.iterdir()]


@pytest.mark.parametrize("input_file", get_files())
def test_files(input_file):
    validate_output(input_file)
