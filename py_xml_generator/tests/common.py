import os
from pathlib import Path

import pytest
import pyxmlgenerator
from xmlschema import XMLSchemaParseError, XMLSchemaModelError, XMLSchema10, XMLSchema11


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


def setup_xmlschema11(filepath):
    try:
        schema = XMLSchema11(filepath)
    except XMLSchemaParseError:
        schema = None
    except XMLSchemaModelError:
        schema = None

    return schema


class Validator:
    def __init__(self, filepath: str):
        try:
            schema = XMLSchema10(filepath)
        except XMLSchemaModelError:
            schema = setup_xmlschema11(filepath)
        except XMLSchemaParseError:
            schema = setup_xmlschema11(filepath)

        self.xmlschema = schema

        if self.xmlschema is None:
            pytest.xfail("Failed to setup schema")

    def print_output(self, xml: str):
        print("XSD")
        print(self.xmlschema.get_text())
        print("XML")
        print(xml)
        print()

    def validate(self, xml: str):
        valid = self.xmlschema.is_valid(xml)

        if not valid:
            self.print_output(xml)

        return valid


def run_generator(xml_generator, filepath) -> str:
    try:
        result: str = xml_generator.generate(filepath)
    except pyxmlgenerator.NoElementsError:
        pytest.xfail("XSD does not contain any elements")
    except pyxmlgenerator.ImplementationError as e:
        pytest.skip(f"Unimplemented feature {e}")
    except pyxmlgenerator.MultipleXSDRootsError:
        pytest.xfail("XSD contains multiple roots")
    except pyxmlgenerator.XSDParserError:
        pytest.xfail("Unable to parse XSD")
    except pyxmlgenerator.InfiniteRecursionError:
        pytest.xfail("Infinite recursion found in XSD")
    except pyxmlgenerator.InvalidXSDNameError:
        pytest.xfail("Invalid XSD naming")
    except pyxmlgenerator.DataTypesFormatError:
        pytest.xfail("Data types format error")
    except pyxmlgenerator.LineEndingsError:
        pytest.xfail("Line endings error")
    except pyxmlgenerator.InvalidXSDVersionError:
        pytest.xfail("Invalid XSD version")
    except pyxmlgenerator.NoIndependentElementsError:
        pytest.xfail("No independent elements in schema")
    except pyxmlgenerator.IncompatiblePatternError:
        pytest.xfail("XSD defines a pattern that is incompatible with the base type")

    return result


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

    validator = Validator(filepath)

    result = run_generator(xml_generator, filepath)

    valid = validator.validate(result)

    os.chdir(cwd)

    assert valid
