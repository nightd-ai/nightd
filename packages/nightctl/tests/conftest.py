"""Pytest fixtures for nightctl tests."""

import pytest
import respx


@pytest.fixture
def respx_mock():
    """Provide a respx mock router for HTTP testing."""
    with respx.mock:
        yield respx
