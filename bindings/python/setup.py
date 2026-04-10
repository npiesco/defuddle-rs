"""Platform-aware binary distribution for the defuddle UniFFI bindings."""

import os
import platform
import sys

from setuptools import Distribution, setup


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


def _native_lib_name() -> str:
    system = platform.system()
    if system == "Windows":
        return "defuddle_rs.dll"
    if system == "Darwin":
        return "libdefuddle_rs.dylib"
    return "libdefuddle_rs.so"


lib_name = _native_lib_name()
package_dir = os.path.join(os.path.dirname(__file__), "defuddle")
lib_path = os.path.join(package_dir, lib_name)

if not os.path.exists(lib_path):
    print(
        f"WARNING: Native library '{lib_name}' not found in {package_dir}. "
        f"Copy it from target/release/ before building the wheel.",
        file=sys.stderr,
    )


setup(
    name="defuddle-py",
    version="0.1.0",
    description="Python bindings for defuddle-rs via UniFFI",
    packages=["defuddle"],
    package_data={"defuddle": [lib_name, "*.py", "*.pyi"]},
    distclass=BinaryDistribution,
    python_requires=">=3.10",
    zip_safe=False,
)
