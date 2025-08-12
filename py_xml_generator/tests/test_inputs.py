import pytest

import xmlschema
import pyxmlgenerator

from pathlib import Path

from xmlschema import XMLSchema


def get_project_root() -> Path:
    """
    Get the project root (defined as location of `.git` directory)

    :return: Path
    """
    return next(
        p for p in Path(__file__).parents
        if (p / '.git').exists()
    )


def get_files() -> list[Path]:
    project_root: Path = get_project_root()
    example_dir: Path = project_root / "examples" / "working"

    return [file for file in example_dir.iterdir()]


@pytest.mark.parametrize("input_file", get_files())
def test_files(input_file):
    with open(input_file, 'r') as f:
        file_data = f.read()

    result: str = pyxmlgenerator.generate(file_data)

    schema: XMLSchema = xmlschema.XMLSchema(file_data)

    print()
    print(schema)
    print(result)

    schema.validate(result)
