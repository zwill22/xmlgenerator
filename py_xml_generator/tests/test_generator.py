import pytest

def test_empty(xml_generator):

    with pytest.raises(RuntimeError) as e:
        xml_generator.generate("")

    assert str(e.value) == "XMLGenerator panic error: Filepath has no parent"
