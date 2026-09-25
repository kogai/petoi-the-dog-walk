import petoi_walk


def test_package_imports_exposes_version() -> None:
    version = petoi_walk.__version__

    assert version == "0.0.0"
