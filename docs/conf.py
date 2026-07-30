import os
import sys

import pyxmlgenerator
import pyxsdtestdata

sys.path.insert(0, os.path.abspath(".."))

project = "pyxmlgenerator"
copyright = "2026, Z M Williams"
author = "Z M Williams"
release = pyxmlgenerator.version()

assert pyxsdtestdata.version() == pyxmlgenerator.version()

extensions = [
    "sphinx.ext.duration",
    "sphinx.ext.intersphinx",
    "sphinx.ext.napoleon",
    "sphinx.ext.autosectionlabel",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "myst_parser",
    "sphinxcontrib_rust",
]

# See docs/compatibility for details on these extensions.
myst_enable_extensions = {
    "attrs_block",
    "colon_fence",
    "html_admonition",
    "replacements",
    "smartquotes",
    "strikethrough",
    "tasklist",
}

rust_crates = {
    "file_to_string": "../reader",
    "xsdtestdata": "../xsd_test_data",
    "xsdvalidator": "../xsd_validator",
    "regextranslator": "../regex_translator",
    "xmlgenerator": "../xml_generator",
}

rust_doc_dir = "crates/"
rust_rustdoc_fmt = "md"

autodoc_typehints = "description"

autodoc_member_order = "bysource"

napoleon_google_docstring = True

intersphinx_mapping = {
    "python": ("https://docs.python.org/3/", None),
    "sphinx": ("https://www.sphinx-doc.org/en/master/", None),
}
intersphinx_disabled_domains = ["std"]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]
html_context = {
    "display_github": True,
    "github_user": "zwill22",
    "github_repo": "xmlgenerator",
    "conf_py_path": "/docs/",
    "github_version": "main",
}

autosectionlabel_prefix_document = True
