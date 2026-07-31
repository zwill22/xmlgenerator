import pytest

from pyxmlgenerator import InvalidPathError

invalid_inputs = ["", "Invalid path format", "/path/in/valid/format/but/does/not/exist"]


@pytest.mark.parametrize(
    "path",
    invalid_inputs,
)
def test_empty(xml_generator, path):

    with pytest.raises(InvalidPathError) as e:
        xml_generator.generate(path)

    assert str(e.value) == "Input path does not exist"
