import pyxmlgenerator

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


def validate_output(input_file: Path | str):
    """
    Validate whether py-xmlgenerator generates output that matches the input schema
    
    :param input_file: Input schema passed to py-xmlgenerator
    
    :raise x
    :return: 
    """
    with open(input_file, 'r') as f:
        file_data = f.read()

    result: str = pyxmlgenerator.generate(file_data)

    schema = XMLSchema(file_data)

    try:
        schema.validate(result)
    except XMLSchemaValidationError as e:
        print()
        print("Result does not match schema:")
        print("XSD")
        print(schema)
        print("XML")
        print(result)
        print()
        raise e
