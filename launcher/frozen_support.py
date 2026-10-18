"""Keep portable user files beside the EXE and native children out of its DLL bundle."""
from contextlib import contextmanager
from pathlib import Path
import os
import sys
import threading

_dll_lock = threading.RLock()

def application_folder(source_file):
    return Path(sys.executable if getattr(sys, 'frozen', False) else source_file).resolve().parent

def _set_dll_directory(path):
    import ctypes
    function = ctypes.WinDLL('kernel32', use_last_error=True).SetDllDirectoryW
    function.argtypes = [ctypes.c_wchar_p]
    function.restype = ctypes.c_int
    if not function(path): raise ctypes.WinError(ctypes.get_last_error())

@contextmanager
def native_process_context():
    if sys.platform != 'win32' or not getattr(sys, 'frozen', False):
        yield
        return
    with _dll_lock:
        _set_dll_directory(None)
        try:
            yield
        finally:
            _set_dll_directory(getattr(sys, '_MEIPASS', None))

def native_environment(env):
    result = dict(env)
    bundle = getattr(sys, '_MEIPASS', None)
    if not getattr(sys, 'frozen', False) or not bundle: return result
    root = Path(bundle).resolve()
    def bundled(entry):
        if not entry: return False
        try: return Path(entry.strip('"')).resolve().is_relative_to(root)
        except (OSError, ValueError): return False
    for key in list(result):
        if key.lower() == 'path':
            result[key] = os.pathsep.join(p for p in result[key].split(os.pathsep) if not bundled(p))
    return result
