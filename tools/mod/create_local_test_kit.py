#!/usr/bin/env python3
"""Export source and two local Windows test BATs; never build or contact GitHub."""
from __future__ import annotations
import argparse
import hashlib
import io
import os
import tempfile
import json
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def batch(action: str) -> bytes:
    setup = ''
    if action == 'build':
        setup = ('where cl >nul 2>&1\nif errorlevel 1 (\n'
                 '  call "%~dp0source\\tools\\mod\\setup_msvc.bat"\n'
                 ')\nwhere cl >nul 2>&1\nif errorlevel 1 (\n'
                 '  echo ERROR: Visual Studio x64 C++ build tools were not found.\n'
                 '  pause\n  exit /b 2\n)\n')
    text = ('@echo off\nsetlocal DisableDelayedExpansion\ncd /d "%~dp0"\n'
            'echo Weapon Evolution v56 local test kit\necho No GitHub operations are performed.\n'
            + setup + 'where py >nul 2>&1\nif not errorlevel 1 (\n'
            '  py -3 "%~dp0source\\tools\\mod\\local_test_update.py" ' + action + ' %*\n'
            ') else (\n  python "%~dp0source\\tools\\mod\\local_test_update.py" ' + action + ' %*\n)\n'
            'set "RESULT=%ERRORLEVEL%"\nif not "%RESULT%"=="0" echo ERROR: Operation failed. Read the message above.\n'
            'pause\nexit /b %RESULT%\n')
    return text.replace('\n','\r\n').encode('utf-8')


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args=parser.parse_args()
    tracked = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
    indexed = json.loads((ROOT/'mod/source-map.json').read_text())['files']
    names=sorted(set(filter(None,tracked)) | {row['path'] for row in indexed})
    if args.output.exists(): raise ValueError('Choose a new ZIP output path.')
    contents={}
    for name in names:
        path=ROOT/name
        if path.suffix.lower() in {'.exe','.dll','.pdb','.zip','.iso','.pyc'}:
            raise ValueError('Generated output in source manifest: '+name)
        contents[name]=path.read_bytes()
    manifest={'patch':'0.1.6.56','files':{name:hashlib.sha256(data).hexdigest() for name,data in contents.items()}}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    prefix='ReRAC_Weapon_Evolution_v56_Local_Test/'
    buffer=io.BytesIO()
    with zipfile.ZipFile(buffer,'w',zipfile.ZIP_DEFLATED) as z:
        for name,data in contents.items(): z.writestr(prefix+'source/'+name,data)
        z.writestr(prefix+'source-files.json',json.dumps(manifest,indent=2)+'\n')
        z.writestr(prefix+'01_UPDATE_BUILD_AND_OPEN.bat',batch('build'))
        z.writestr(prefix+'02_OPEN_TEST_LAUNCHER.bat',batch('open'))
        z.writestr(prefix+'README_FIRST.txt',(ROOT/'docs/mod/LOCAL_TEST_KIT.md').read_bytes())
    data=buffer.getvalue()
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        if z.testzip() is not None: raise ValueError('ZIP integrity check failed.')
    descriptor, temporary=tempfile.mkstemp(prefix='.local-test-kit-',dir=args.output.parent)
    try:
        with os.fdopen(descriptor,'wb') as stream:
            stream.write(data);stream.flush();os.fsync(stream.fileno())
        os.replace(temporary,args.output)
    finally:
        Path(temporary).unlink(missing_ok=True)
    digest=hashlib.sha256(data).hexdigest()
    Path(str(args.output)+'.sha256').write_text(digest+'  '+args.output.name+'\n')
    print(f'{args.output}\n{len(contents)} source files; {args.output.stat().st_size} bytes\nSHA256 {digest}')


if __name__=='__main__': main()
