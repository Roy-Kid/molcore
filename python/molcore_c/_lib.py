import ctypes
import os
import sys


def _default_lib_names():
    if sys.platform.startswith("linux"):
        return ["libmolomni_c.so", "libmolomni-c.so", "libmolomni_c.dylib"]
    elif sys.platform == "darwin":
        return ["libmolomni_c.dylib", "libmolomni-c.dylib", "libmolomni_c.so"]
    elif sys.platform == "win32":
        return ["molomni_c.dll", "molomni-c.dll"]
    else:
        return ["libmolomni_c.so"]


def _find_library():
    # Prefer local development build paths
    roots = []
    here = os.path.abspath(os.path.dirname(__file__))
    roots.append(os.path.join(here, "..", "..", "target", "debug"))
    roots.append(os.path.join(here, "..", "..", "target", "release"))
    roots.append(here)

    names = _default_lib_names()
    for root in roots:
        for name in names:
            path = os.path.abspath(os.path.join(root, name))
            if os.path.exists(path):
                return path
    # Fallback to system loader search
    return None


_path = _find_library()
if _path is None:
    # This will still likely fail, but gives ctypes a chance to locate it
    _path = _default_lib_names()[0]

lib = ctypes.CDLL(_path)
