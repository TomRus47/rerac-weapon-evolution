"""Author-only Windows build. Players need neither Python nor Tcl/Tk installed."""
from __future__ import annotations
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import venv
import launcher_core as core
from icon_assets import preserve_icon

PYINSTALLER_VERSION = '6.22.3'
EXE_NAME = 'ReRAC_Weapon_Evolution.exe'
SOURCES = ('windows_entry.py','launcher.py','launcher_core.py','schema.py','migrate_ini.py','frozen_support.py','icon_assets.py','i18n.py')

def source_digest(kit):
    h=hashlib.sha256()
    for name in SOURCES:
        h.update(name.encode());h.update(b'\0');h.update((kit/name).read_bytes());h.update(b'\0')
    icon=kit/'launcher.ico'
    h.update(b'launcher.ico\0')
    h.update(icon.read_bytes() if icon.is_file() else b'<default icon>')
    h.update(PYINSTALLER_VERSION.encode())
    h.update(Path(__file__).read_bytes())
    return h.hexdigest()

def icon_file(kit):
    path=kit/'launcher.ico'
    if not path.is_file():return None
    data=path.read_bytes()
    if len(data)<6:raise ValueError('launcher.ico is not a valid ICO file.')
    reserved,kind,count=struct.unpack_from('<HHH',data)
    end=6+16*count
    if reserved!=0 or kind!=1 or count==0 or len(data)<end:
        raise ValueError('launcher.ico must be a real Windows ICO file, not a renamed PNG or JPG.')
    for i in range(count):
        size,offset=struct.unpack_from('<II',data,6+16*i+8)
        if size==0 or offset<end or offset+size>len(data):
            raise ValueError('launcher.ico contains an incomplete image.')
    return path

def windows_x64(path):
    with path.open('rb') as stream:
        header=stream.read(64)
        if len(header)<64 or header[:2]!=b'MZ':raise ValueError('Invalid launcher EXE.')
        stream.seek(struct.unpack_from('<I',header,60)[0]);pe=stream.read(6)
    if len(pe)!=6 or pe[:4]!=b'PE\0\0' or struct.unpack_from('<H',pe,4)[0]!=0x8664:
        raise ValueError('The launcher must be a Windows x64 executable.')

def smoke_test(exe):
    windows_x64(exe)
    with tempfile.TemporaryDirectory(prefix='rerac_exe_check_') as temporary:
        report=Path(temporary)/'check.json'
        subprocess.run([str(exe.resolve()),'--self-test',str(report)],check=True,timeout=120)
        value=json.loads(report.read_text(encoding='utf-8'))
    if (not value.get('frozen') or not value.get('gui_ok') or value.get('launcher_version')!=core.VERSION
            or Path(value.get('application_folder','')).resolve()!=exe.parent.resolve()
            or not value.get('tcl_version') or not value.get('tk_version')):
        raise ValueError('Standalone launcher verification failed: invalid Python/Tk resources or application folder.')
    return value

def ensure_tools(kit):
    if sys.platform!='win32':raise ValueError('Build the EXE on the Windows authoring PC using BUILD_FINAL_WINDOWS_EXE.bat.')
    if sys.version_info<(3,10) or struct.calcsize('P')!=8:
        raise ValueError('The authoring PC requires 64-bit Python 3.10+. Players do not need Python.')
    try:
        import tkinter as tk
        root=tk.Tk();root.withdraw();root.update_idletasks();root.destroy()
    except Exception as error:
        raise ValueError('Python on the authoring PC must include tcl/tk and IDLE. Players do not need to install them.') from error
    folder=kit/'.launcher-tools/venv'
    python=folder/'Scripts/python.exe'
    if not python.is_file():
        print('Preparing the private launcher build environment…',flush=True)
        venv.EnvBuilder(with_pip=True).create(folder)
    version=subprocess.run([str(python),'-m','PyInstaller','--version'],capture_output=True,text=True)
    if version.returncode or version.stdout.strip()!=PYINSTALLER_VERSION:
        print('Installing launcher build tools on the authoring PC…',flush=True)
        subprocess.run([str(python),'-m','pip','install','--disable-pip-version-check','--only-binary=:all:',
                        'PyInstaller=='+PYINSTALLER_VERSION],check=True,cwd=kit)
    return python

def collect_licenses(python, output):
    # Keep the actual installed license texts alongside the distributed runtime.
    code='''from pathlib import Path
import importlib.metadata as m,json,sys,tkinter as tk
root=tk.Tk();root.withdraw()
paths=[Path(sys.base_prefix)/'LICENSE.txt',Path(root.tk.call('info','library'))/'license.terms']
tk_library=Path(str(root.tk.eval('set tk_library')))
paths.append(tk_library/'license.terms');root.destroy()
for f in m.files('pyinstaller') or []:
 if 'license' in str(f).lower() or Path(str(f)).name.lower().startswith('copying'):
  p=Path(m.distribution('pyinstaller').locate_file(f))
  if p.is_file():paths.append(p)
print(json.dumps([{'name':str(i)+'_'+p.name,'text':p.read_text(encoding='utf-8',errors='replace')} for i,p in enumerate(paths) if p.is_file()]))
'''
    result=subprocess.run([str(python),'-c',code],capture_output=True,text=True,encoding='utf-8',check=True)
    output.mkdir(parents=True,exist_ok=True)
    for item in json.loads(result.stdout):core.atomic_write(output/item['name'],item['text'].encode('utf-8'))

def build(kit):
    kit=kit.resolve()
    if sys.platform!='win32':raise ValueError('Run BUILD_FINAL_WINDOWS_EXE.bat on the Windows authoring PC.')
    preserve_icon(kit)
    icon=icon_file(kit)
    icon_args=['--icon',str(icon),'--add-data',str(icon)+':.'] if icon else []
    fingerprint=source_digest(kit)
    output=kit/'launcher-bin'/EXE_NAME
    metadata=output.parent/'launcher-build.json'
    if output.is_file() and metadata.is_file():
        try:
            saved=json.loads(metadata.read_text(encoding='utf-8'))
            if (saved.get('sources_sha256')==fingerprint
                    and saved.get('exe_sha256')==hashlib.sha256(output.read_bytes()).hexdigest()):
                smoke_test(output)
                print('Launcher EXE is already up to date and verified.',flush=True)
                return output
        except (ValueError,OSError,subprocess.SubprocessError):pass
    python=ensure_tools(kit)
    build_root=kit/'.launcher-build';build_root.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='build_',dir=build_root) as temporary:
        stage=Path(temporary)
        print('Building the standalone Windows launcher…',flush=True)
        subprocess.run([str(python),'-m','PyInstaller','--noconfirm','--clean','--onefile','--windowed','--noupx',*icon_args,
            '--name',Path(EXE_NAME).stem,'--distpath',str(stage/'dist'),'--workpath',str(stage/'work'),
            '--specpath',str(stage),'--paths',str(kit),str(kit/'windows_entry.py')],check=True,cwd=kit)
        built=stage/'dist'/EXE_NAME
        info=smoke_test(built)
        licenses=stage/'licenses';collect_licenses(python,licenses)
        output.parent.mkdir(exist_ok=True)
        core.atomic_write(output,built.read_bytes())
        shutil.copytree(licenses,output.parent/'licenses',dirs_exist_ok=True)
        core.atomic_write(metadata,(json.dumps({'launcher_version':core.VERSION,'sources_sha256':fingerprint,
            'exe_sha256':hashlib.sha256(output.read_bytes()).hexdigest(),'self_test':info},indent=2)+'\n').encode('utf-8'))
    print('Launcher ready: '+str(output),flush=True)
    return output

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder',type=Path,default=core.KIT)
    args=parser.parse_args()
    try:build(args.folder);return 0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        print('ERROR: '+str(error),file=sys.stderr);return 2

if __name__=='__main__':raise SystemExit(main())
