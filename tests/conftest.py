"""Shared pytest setup. See docs/rules/testing.md."""

import os

import pytest
from hypothesis import settings

settings.register_profile("ci", derandomize=True, print_blob=True)
settings.register_profile("dev", max_examples=50)
settings.load_profile(os.environ.get("HYPOTHESIS_PROFILE", "dev"))

_OPT_IN = {
    "jev": "--run-jev",
    "hardware": "--run-hardware",
}


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption("--run-jev", action="store_true", help="run tests that call the real Jev API")
    parser.addoption(
        "--run-hardware", action="store_true", help="run tests that drive a real Bittle"
    )


def pytest_collection_modifyitems(config: pytest.Config, items: list[pytest.Item]) -> None:
    for item in items:
        for marker, option in _OPT_IN.items():
            if marker in item.keywords and not config.getoption(option):
                item.add_marker(pytest.mark.skip(reason=f"needs {option}"))
