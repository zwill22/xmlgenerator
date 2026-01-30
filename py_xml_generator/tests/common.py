import os
from pathlib import Path

from pyxmlgenerator import MultipleXSDRootsError
from xmlschema import XMLSchema, XMLSchemaValidationError, XMLSchemaParseError


def get_project_root() -> Path:
    """
    Get the project root (defined as location of `.git` directory)

    :return: Path
    """
    return next(
        p for p in Path(__file__).parents
        if (p / '.git').exists()
    )


def identity(file: Path, parent_path: Path) -> str:
    full_path = str(file)
    root_path = str(parent_path)

    out = full_path.replace(root_path, "")

    if out.startswith("/"):
        return out[1:]

    return out


def validate_output(xml_generator, input_file: Path | str):
    """
    Validate whether py-xmlgenerator generates output that matches the input schema

    :param xml_generator: XMLGenerator fixture
    :param input_file: Input schema passed to py-xmlgenerator
    
    :raise x
    :return: 
    """

    filepath = str(input_file)
    file_dir = Path(input_file).parent

    cwd = Path.cwd()
    os.chdir(file_dir)

    try:
        schema = XMLSchema(filepath)
    except XMLSchemaParseError:
        print("XMLSchema is unable to parse the schema, skipping test")
        return

    result: str = xml_generator.generate(filepath)

    try:
        schema.validate(result)
    except XMLSchemaValidationError as e:
        print()
        print("Result does not match schema:")
        print("XSD")
        print(schema.get_text())
        print("XML")
        print(result)
        print()
        raise e
    finally:
        os.chdir(cwd)
