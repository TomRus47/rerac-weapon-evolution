"""Export an integrity-checked standalone v136 source test kit; no Git required."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import zipfile

SOURCE = Path(__file__).resolve().parents[2]
NAME = 'ReRAC_Weapon_Evolution_v136_Gadget_Combat_Test'
PATCH = '0.1.6.136'


def batch(action):
    extra = ''  # Rebuild the editor too: Freezer recharge adds a visible setting.
    text = ('@echo off\nsetlocal\ncd /d "%~dp0"\n'
        'echo Weapon Evolution v136 standalone local test. No GitHub operations.\n'
        'where py >nul 2>&1\nif errorlevel 1 goto python_fallback\n'
        'py -3 "%~dp0source\\tools\\mod\\runtime038_test.py" ' + action + extra + '\n'
        'set "WE_TEST_RESULT=%ERRORLEVEL%"\ngoto finished\n'
        ':python_fallback\npython "%~dp0source\\tools\\mod\\runtime038_test.py" ' + action + extra + '\n'
        'set "WE_TEST_RESULT=%ERRORLEVEL%"\n:finished\necho.\n'
        'if not "%WE_TEST_RESULT%"=="0" echo Operation failed. Read the error and logs above.\n'
        'pause\nexit /b %WE_TEST_RESULT%\n')
    return text.replace('\n', '\r\n').encode('ascii')


def create(destination, inventory=None):
    from verified_artifacts import validate_source_apis
    validate_source_apis(SOURCE)
    inventory = Path(inventory) if inventory else SOURCE.parent / 'source-files.json'
    manifest = json.loads(inventory.read_text(encoding='utf-8'))
    if manifest.get('patch') != PATCH or manifest.get('runtime_base') != '0.3.8-alpha':
        raise ValueError('Export only the verified v136 / ReRAC 0.3.8-alpha source.')
    required = {'Cargo.toml', 'Cargo.lock', 'AGENTS.md', 'mod/development.json',
        'tools/mod/runtime038_test.py', 'tools/mod/rust_build_tools.py',
        'tools/mod/windows_build_env.py', 'tools/mod/verified_artifacts.py',
        'tools/mod/rust_download.py', 'launcher/launcher_core.py', 'launcher/schema.py'}
    if not isinstance(manifest.get('files'), dict) or not required.issubset(manifest['files']):
        raise ValueError('Incomplete standalone source inventory.')
    entries = {}
    for name, digest in manifest['files'].items():
        rel = PurePosixPath(name)
        if rel.is_absolute() or '..' in rel.parts or '\\' in name or ':' in name:
            raise ValueError('Unsafe inventory path: ' + name)
        path = SOURCE.joinpath(*rel.parts)
        if path.is_symlink() or not path.resolve().is_relative_to(SOURCE.resolve()):
            raise ValueError('Source path leaves the verified tree: ' + name)
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != digest:
            raise ValueError('Source integrity check failed: ' + name)
        entries['source/' + name] = data
    entries['source-files.json'] = (json.dumps(manifest, indent=2) + '\n').encode()
    entries['README_FIRST.md'] = (SOURCE / 'docs/mod/LOCAL_TEST_V136.md').read_bytes()
    entries['LICENSE'] = (SOURCE / 'LICENSE').read_bytes()
    for name, action in [('01_COMPILE_AND_UPDATE.bat', 'build'),
                         ('02_OPEN_UPDATED_LAUNCHER.bat', 'open'),
                         ('03_CREATE_PLAYABLE_ZIP.bat', 'package')]:
        entries[name] = batch(action)
    destination = Path(destination).resolve()
    destination.mkdir(parents=True, exist_ok=True)
    archive = destination / (NAME + '.zip')
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(entries.items()):
            z.writestr(NAME + '/' + name, data)
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:
            raise ValueError('ZIP integrity check failed.')
        for name, digest in manifest['files'].items():
            if hashlib.sha256(z.read(NAME + '/source/' + name)).hexdigest() != digest:
                raise ValueError('Archived source differs: ' + name)
    print('Standalone v136 source kit: ' + str(archive))
    return archive


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--inventory', type=Path)
    args = parser.parse_args()
    create(args.destination, args.inventory)
