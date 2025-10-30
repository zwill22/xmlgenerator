import re

with open("output.txt", 'r') as f:
    out: list[str] = f.readlines()

impl = set()
invalid_data_type = set()
independent = set()
dependent = set()
unknown_values = set()
unknown_value_positions = set()
unknown_value_elements = set()
invalid_version = set()
infinite_recursion = 0
resource_locations = set()
resource_currents = set()
empty_xsd = 0
invalid_file = 0
attribute_errors = set()
other = set()


def get_position(data_list: list[str]) -> int:
    assert len(data_list) == 3
    pos_str = data_list[1].split("=")
    assert len(pos_str) == 2
    assert pos_str[0].strip() == "position"
    return int(pos_str[1])


def get_element(data_list: list[str]) -> str:
    assert len(data_list) == 3
    el_str = data_list[2].split("=")
    assert len(el_str) == 2
    assert el_str[0].strip() == "element"
    return el_str[1].strip()


for item in out:
    if "Implementation error: not implemented: " in item:
        impl.add(item.replace("Implementation error: not implemented: ", ""))
    elif "DataType not found: " in item:
        invalid_data_type.add(item.replace("DataType not found: ", ""))
    elif "Independent element: " in item:
        result = re.findall(r"Some\(\"(.*?)\"\)", item)
        independent.add(result[0])
    elif "Dependent element: " in item:
        result = re.findall(r"Some\(\"(.*?)\"\)", item)
        dependent.add(result[0])
    elif "XSD parser error: XML Error: Unknown or invalid value: " in item:
        result = item.replace("XSD parser error: XML Error: Unknown or invalid value:", "").split(";")
        assert len(result) == 3
        string_val = result[0]
        position = get_position(result)
        element = get_element(result)

        unknown_values.add(string_val)
        unknown_value_elements.add(element)
        unknown_value_positions.add(position)
    elif "XSD parser error: Unable to resolve requested resource: " in item:
        result = re.findall(r"Location=(.*)Current=(.*)", item)[0]

        location = result[0]
        current = result[1]

        resource_locations.add(location)
        resource_currents.add(current)
    elif "Infinite recursion detected" in item:
        infinite_recursion += 1
    elif "XSD does not contain any elements" in item:
        empty_xsd += 1
    elif "Invalid XSD version: " in item:
        invalid_version.add(item.replace("Invalid XSD version: ", ""))
    elif "XSD parser error: Resolver Error: No such file or directory (os error 2)" in item:
        invalid_file += 1
    elif "XSD parser error: XML Error: Attribute Error: " in item:
        attribute_errors.add(item.replace("XSD parser error: XML Error: Attribute Error: ", ""))
    elif item.strip() != "":
        other.add(item)


def print_num(n: int, title: str, **kwargs):
    if n == 0:
        return

    t = 32 - len(title)
    blank = ""
    print(f"{title}:{blank:<{t}}{n:2d}", **kwargs)


def print_number(dataset: set, title: str, **kwargs):
    n = len(dataset)
    print_num(n, title, **kwargs)


def print_unknown(values: set[str], positions: set[int], elements: set[str]):
    n_unknown = len(values)
    n_positions = len(positions)
    n_elements = len(elements)
    if n_unknown == 0:
        assert n_elements == 0
        assert n_positions == 0
        return

    print_number(values, "Unknown values", end='\t')
    print(f"(positions: {n_positions}, elements: {n_elements})")


def print_resource_errors(locations: set[str], currents: set[str]):
    if len(locations) != len(currents):
        raise ValueError("Invalid resource errors")

    print_number(locations, "Invalid resources")

print_number(impl, "Implementation errors")
print_number(invalid_data_type, "Invalid data types")
print_number(independent, "Independent elements")
print_number(dependent, "Dependent errors")
print_unknown(unknown_values, unknown_value_positions, unknown_value_elements)
print_num(infinite_recursion, "Infinite recursion")
print_resource_errors(resource_locations, resource_currents)
print_number(attribute_errors, "Attribute errors")
print_num(empty_xsd, "XSD containing no elements")
print_number(invalid_version, "Invalid versions")
print_num(invalid_file, "Invalid files")

def print_results(data: set[str], title: str):
    n = len(data)
    if n == 0:
        return

    string = f"{n:2d} {title} "
    under = len(string) * "="
    print()
    print(string)
    print(under)
    for val in data:
        print(end='\t')
        print(val.strip())
    print()

print()
print_results(impl, "Implementation errors")
print_results(invalid_data_type, "Invalid data types")
print_results(independent, "Independent elements")
print_results(dependent, "Dependent elements")
print_results(unknown_value_elements, "Unknown value locations")
print_results(invalid_version, "Invalid versions")
print_results(resource_locations, "Invalid Resource Locations")
print_results(resource_currents, "Invalid Resource CurrentLocations")
print_results(attribute_errors, "Attribute errors")


print("-" * 128)
for item in other:
    print(item, end='')
