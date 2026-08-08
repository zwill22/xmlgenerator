import os

# On Windows systems, Python uses DLL paths when resolving 
# dependencies in Python extension modules, see:
# https://docs.python.org/3/library/os.html#os.add_dll_directory
if os.name == "nt":
    for path in os.environ["PATH"].split(";"):
        try:
            os.add_dll_directory(str(path)) 
        except FileNotFoundError:
            pass
    