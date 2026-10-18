"""PyInstaller entry point. Player data always lives beside the executable."""
from pathlib import Path
import json
import os
import sys
import traceback

def self_test(destination):
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox, simpledialog, colorchooser
    import launcher_core as core
    import migrate_ini
    import schema
    root = tk.Tk()
    try:
        root.withdraw()
        ttk.Label(root, text='ReRAC').pack()
        root.update_idletasks()
        result = {'frozen': bool(getattr(sys,'frozen',False)), 'launcher_version': core.VERSION,
                  'application_folder': str(core.KIT), 'gui_ok': True,
                  'tcl_version': root.tk.call('info','patchlevel'), 'tk_version': str(tk.TkVersion)}
    finally:
        root.destroy()
    core.atomic_write(Path(destination), (json.dumps(result,indent=2)+'\n').encode('utf-8'))
    return 0

def entry():
    # Windowed executables have no console streams; library notices remain harmless.
    for name in ('stdout','stderr'):
        if getattr(sys,name) is None: setattr(sys,name,open(os.devnull,'w',encoding='utf-8'))
    if len(sys.argv)==3 and sys.argv[1]=='--self-test': return self_test(sys.argv[2])
    import launcher
    return launcher.main()

if __name__ == '__main__':
    try:
        raise SystemExit(entry())
    except Exception:
        from frozen_support import application_folder
        folder = application_folder(__file__)
        message = traceback.format_exc()
        try: (folder/'launcher-error.log').write_text(message,encoding='utf-8')
        except OSError: pass
        if sys.platform=='win32':
            import ctypes
            ctypes.windll.user32.MessageBoxW(None,'Unable to open the launcher.\nSee launcher-error.log beside the executable for details.','ReRAC Weapon Evolution',0x10)
        raise SystemExit(2)
