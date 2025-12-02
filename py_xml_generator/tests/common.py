import os
from pathlib import Path
from xmlschema import XMLSchema, XMLSchemaValidationError


def get_project_root() -> Path:
    """
    Get the project root (defined as location of `.git` directory)

    :return: Path
    """
    return next(
        p for p in Path(__file__).parents
        if (p / '.git').exists()
    )


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

    result: str = xml_generator.generate(filepath)

    schema = XMLSchema(filepath)

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
