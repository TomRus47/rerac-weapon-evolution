#!/usr/bin/env python3
"""Export a self-contained local test kit; never access GitHub or player files."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]
NAME = 'ReRAC_Weapon_Evolution_v124_Amoeboids_Crates_Beams_Test'


def batch(action):
    extra = ' --runtime-only' if action == 'build' else ''
    text = ('@echo off\nsetlocal\ncd /d "%~dp0"\n'
        'echo Weapon Evolution v124 local test. No GitHub operations.\n'
        'py -3 "source\\tools\\mod\\runtime038_test.py" ' + action + extra + '\n'
        'set "WE_TEST_RESULT=%ERRORLEVEL%"\n'
        'echo.\nif not "%WE_TEST_RESULT%"=="0" echo Operation failed. Read the error and logs above.\n'
        'pause\nexit /b %WE_TEST_RESULT%\n')
    return text.replace('\n','\r\n').encode('ascii')


def create(destination):
    from verified_artifacts import validate_source_apis
    validate_source_apis(ROOT)
    listing = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT)
    paths = sorted(n.decode('utf-8') for n in listing.split(b'\0') if n)
    identity = json.loads((ROOT/'mod/development.json').read_text())
    if identity['patch'] != '0.1.6.124':
        raise ValueError('Export only the reviewed v124 source.')
    entries = {}
    for name in paths:
        path = ROOT/name
        if path.is_symlink() or not path.is_file():
            raise ValueError('Invalid source path: '+name)
        entries['source/'+name] = path.read_bytes()
    manifest = {'patch':identity['patch'],'runtime_base':'0.3.8-alpha',
        'files':{name:hashlib.sha256(entries['source/'+name]).hexdigest() for name in paths}}
    entries['source-files.json'] = (json.dumps(manifest,indent=2)+'\n').encode()
    entries['README_FIRST.txt'] = (ROOT/'docs/mod/LOCAL_TEST_V124.md').read_bytes()
    entries['LICENSE'] = (ROOT/'LICENSE').read_bytes()
    for name,action in [('01_COMPILE_AND_UPDATE.bat','build'),
            ('02_OPEN_UPDATED_LAUNCHER.bat','open'),('03_CREATE_PLAYABLE_ZIP.bat','package')]:
        entries[name] = batch(action)
    destination = destination.resolve()
    destination.mkdir(parents=True,exist_ok=True)
    archive = destination/(NAME+'.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for name,data in sorted(entries.items()):
            z.writestr(NAME+'/'+name,data)
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:
            raise ValueError('ZIP validation failed.')
        for name,digest in manifest['files'].items():
            if hashlib.sha256(z.read(NAME+'/source/'+name)).hexdigest()!=digest:
                raise ValueError('Archived source differs: '+name)
    print(archive)
    print('SHA256: '+hashlib.sha256(archive.read_bytes()).hexdigest())
    print('Verified source files: '+str(len(paths)))
    return archive


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination',type=Path)
    create(parser.parse_args().destination)
