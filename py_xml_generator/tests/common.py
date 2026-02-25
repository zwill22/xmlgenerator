import os
from pathlib import Path

import pytest
import pyxmlgenerator
from xmlschema import XMLSchema, XMLSchemaValidationError, XMLSchemaParseError, XMLSchemaModelError
from xmlschema.exceptions import XMLResourceParseError


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


def print_output(schema: XMLSchema, result: str):
    print("XSD")
    print(schema.get_text())
    print("XML")
    print(result)
    print()


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
    except XMLSchemaModelError:
        pytest.skip("Invalid schema")
    except XMLSchemaParseError:
        pytest.skip("Unable to parse schema")

    try:
        result: str = xml_generator.generate(filepath)
    except pyxmlgenerator.NoElementsError:
        pytest.skip("XSD does not contain any elements")
    except pyxmlgenerator.ImplementationError as e:
        pytest.xfail(f"Unimplemented feature {e}")
    except pyxmlgenerator.MultipleXSDRootsError:
        pytest.skip("XSD contains multiple roots")
    except pyxmlgenerator.XSDParserError:
        pytest.skip("Unable to parse XSD")
    except pyxmlgenerator.InfiniteRecursionError:
        pytest.skip("Infinite recursion found in XSD")
    except pyxmlgenerator.InvalidXSDNameError:
        pytest.skip("Invalid XSD naming")
    except pyxmlgenerator.DataTypesFormatError:
        pytest.skip("Data types format error")
    except pyxmlgenerator.LineEndingsError:
        pytest.skip("Line endings error")
    except pyxmlgenerator.InvalidXSDVersionError:
        pytest.skip("Invalid XSD version")
    except pyxmlgenerator.NoIndependentElementsError:
        pytest.skip("No independent elements in schema")

    try:
        schema.validate(result)
    except XMLSchemaValidationError as e:
        print()
        print("Result does not match schema:")
        print_output(schema, result)
        raise e
    except XMLResourceParseError as e:
        print()
        print("Unable to parse XML:")
        print_output(schema, result)
        raise e
    finally:
        os.chdir(cwd)
