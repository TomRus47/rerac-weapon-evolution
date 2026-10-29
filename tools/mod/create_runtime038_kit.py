"""Export complete reviewed source and local Windows update/package entry points."""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]
NAME = 'ReRAC_Weapon_Evolution_v114_Pickup_Completion_Local_Test'
README = '''Weapon Evolution v114 / ReRAC 0.3.8-alpha local source kit

Extract the entire ZIP to a new writable folder, close the game, then run
01_COMPILE_AND_UPDATE.bat. Select the EXISTING playable mod folder containing
your launcher EXE, weapons.ini and runtime/rerac.exe, not this kit or source.
Python 3.10+, Rust/Cargo and installed Visual Studio C++ x64 tools/Windows SDK
are needed for compilation. The script initializes Visual Studio automatically.
Uncached dependencies require internet; a cold build can take a long time.

Compilation uses these bundled merged sources. The old source checkout is not
patched. Its target folder may be reused only as a Cargo cache. An explicit
cache can be chosen with --target-dir "C:\\ReRAC-Mod\\target-038".
Tests/build/identity checks complete before installed files change. The EXE and
matching shaders are backed up and replaced together. A failed installation
rolls back changed files. Your INI, presets, saves and launcher/icon are retained.
The existing 0.29.3 launcher is compatible; no GUI rebuild is needed for these fixes.

02_OPEN_UPDATED_LAUNCHER.bat verifies and reopens the same installation.
03_CREATE_PLAYABLE_ZIP.bat creates a player ZIP under this kit's dist folder,
using the existing launcher and current weapons.ini, without personal presets,
save files, paths or disc assets. Players do not need Python to run that EXE.
They still need their own extracted game data. Keep the player ZIP together.

Logs: logs/runtime038-*.log and logs/update-error.log in this kit.
Backups: backup/runtime_038/<timestamp> in the playable folder. restore.json maps
numbered backup files to their original paths; retain it for manual recovery.
The BAT windows pause on completion, including errors. No GitHub operations occur.

Test a randomized Gold Bolt pickup on foot: the bolt must animate once, disappear,
and return the normal camera, hand item, HUD and movement. Also collect a gadget
with its native scene and a weapon. Jingles must finish and resume the held music.
Notes: docs/mod/PICKUP_COMPLETION_V114.md. Existing v113 XP fixes and the complete
ReRAC 0.3.8 integration remain. No GUI rebuild or INI migration is needed.

Weapon Evolution is an independent mod by TomRus47. The author is not a ReRAC
developer and owns no original-game rights or assets. Thanks to ReRAC's developers
for their work that makes this mod possible. Upstream licenses and credits apply.
'''


def batch(action):
    text = ('@echo off\nsetlocal DisableDelayedExpansion\ncd /d "%~dp0"\n'
        'where py >nul 2>&1\nif errorlevel 1 goto usepython\n'
        'py -3 "%~dp0source\\tools\\mod\\runtime038_test.py" ' + action + ' %*\n'
        'set "result=%errorlevel%"\ngoto done\n:usepython\n'
        'python "%~dp0source\\tools\\mod\\runtime038_test.py" ' + action + ' %*\n'
        'set "result=%errorlevel%"\n:done\necho.\necho Operation exited with code %result%.\n'
        'if not "%result%"=="0" echo Read the error above before closing this window.\n'
        'pause\nexit /b %result%\n')
    return text.replace('\n', '\r\n').encode()


def create(output):
    names = sorted({p.decode() for p in subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0') if p})
    entries, hashes = {}, {}
    for name in names:
        path = ROOT / name
        if not path.is_file() or path.is_symlink():
            raise ValueError('Invalid source export entry: ' + name)
        data = path.read_bytes()
        entries['source/' + name] = data
        hashes[name] = hashlib.sha256(data).hexdigest()
    entries['source-files.json'] = (json.dumps({'patch': '0.1.6.114',
        'runtime_base': '0.3.8-alpha', 'files': hashes}, indent=2) + '\n').encode()
    entries['README_FIRST.txt'] = README.encode()
    for filename, action in [('01_COMPILE_AND_UPDATE.bat', 'build'),
        ('02_OPEN_UPDATED_LAUNCHER.bat', 'open'), ('03_CREATE_PLAYABLE_ZIP.bat', 'package')]:
        entries[filename] = batch(action)
    output.mkdir(parents=True, exist_ok=True)
    archive = output / (NAME + '.zip')
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(entries.items()):
            z.writestr(NAME + '/' + name, data)
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:
            raise ValueError('Archive integrity check failed.')
        for name, digest in hashes.items():
            if hashlib.sha256(z.read(NAME + '/source/' + name)).hexdigest() != digest:
                raise ValueError('Archive source mismatch: ' + name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix('.zip.sha256').write_text(digest + '  ' + archive.name + '\n')
    print(str(archive.resolve()))
    return archive


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    create(parser.parse_args().output)
