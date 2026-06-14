"""Sphinx configuration for conda-express documentation."""

from importlib import metadata
import os
from pathlib import Path
import re
import subprocess

project = html_title = "conda-express"
copyright = "2026, Jannis Leidel"
author = "Jannis Leidel"

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_TAG_PATTERN = re.compile(
    r"^v?(?P<release>\d+\.\d+\.\d+(?:\.post\d+)?)(?:\.dev\d+)?$"
)


def normalize_release_tag(tag):
    match = RELEASE_TAG_PATTERN.match(tag.strip())
    if match is None:
        raise RuntimeError(f"invalid conda-express release tag: {tag!r}")
    return match.group("release")


def latest_release_tag():
    override = os.environ.get("CONDA_EXPRESS_DOCS_RELEASE")
    if override:
        return normalize_release_tag(override)

    try:
        result = subprocess.run(
            [
                "git",
                "describe",
                "--tags",
                "--abbrev=0",
                "--match",
                "[0-9]*",
                "--match",
                "v[0-9]*",
            ],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return normalize_release_tag(metadata.version("conda-express"))
    return normalize_release_tag(result.stdout)


release = latest_release_tag()
version = release.split(".post", 1)[0]

extensions = [
    "myst_parser",
    "sphinx.ext.intersphinx",
    "sphinx_copybutton",
    "sphinx_design",
    "sphinx_reredirects",
    "sphinx_sitemap",
]

myst_enable_extensions = [
    "colon_fence",
    "deflist",
    "fieldlist",
    "tasklist",
]

versioned_example_values = {
    "conda_runtime_version": version,
    "conda_express_release": release,
    "conda_express_post_release_example": f"{version}.post1",
    "setup_cx_action": f"jezdez/conda-express/.github/actions/setup-cx@{release}",
}

html_theme = "conda_sphinx_theme"

html_theme_options = {
    "icon_links": [
        {
            "name": "GitHub",
            "url": "https://github.com/jezdez/conda-express",
            "icon": "fa-brands fa-square-github",
            "type": "fontawesome",
        },
        {
            "name": "PyPI",
            "url": "https://pypi.org/project/conda-express/",
            "icon": "fa-brands fa-python",
            "type": "fontawesome",
        },
    ],
}

html_context = {
    "github_user": "jezdez",
    "github_repo": "conda-express",
    "github_version": "main",
    "doc_path": "docs",
}

html_static_path = ["_static"]
html_extra_path = ["../scripts/get-cx.sh", "../scripts/get-cx.ps1"]
html_css_files = ["css/custom.css"]

html_baseurl = "https://jezdez.github.io/conda-express/"

intersphinx_mapping = {
    "conda-ship": ("https://jezdez.github.io/conda-ship/", None),
}

exclude_patterns = ["_build"]


def replace_versioned_example_values(app, docname, source):
    """Render configured release values in prose and code examples."""
    text = source[0]
    for name, value in versioned_example_values.items():
        text = text.replace("{{ " + name + " }}", value)
    source[0] = text


def setup(app):
    app.connect("source-read", replace_versioned_example_values)
