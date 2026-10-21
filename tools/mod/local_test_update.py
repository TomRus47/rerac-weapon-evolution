#!/usr/bin/env python3
"""Install a reviewed source snapshot locally, reuse Cargo's target and open the editor.

No Git operations, remote builds, releases or launcher EXE rebuilds are used.
The bundle contains a complete source overlay rather than brittle text patches.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys

PAYLOAD = Path(__file__).resolve().parents[2]
BUNDLE = PAYLOAD.parent
sys.path.insert(0, str(PAYLOAD / 'launcher'))
import editabile
import launcher_core as core
from migrate_ini import convert


def source_files(payload: Path, manifest: Path) -> list[Path]:
    """Verify every delivered file before changing either destination folder."""
    records = json.loads(manifest.read_text(encoding='utf-8'))
    if records.get('patch') != '0.1.6.59':
        raise ValueError('The source manifest is not Weapon Evolution v59.')
    paths = []
    for name, expected in records['files'].items():
        relative = PurePosixPath(name)
        if relative.is_absolute() or not relative.parts or '..' in relative.parts or '\\' in name or ':' in name:
            raise ValueError('Unsafe source manifest path: ' + name)
        path = payload.joinpath(*relative.parts)
        if not path.resolve().is_relative_to(payload.resolve()) or path.is_symlink():
            raise ValueError('Source file escapes the bundle: ' + name)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError('Damaged source file; extract the complete ZIP again: ' + name)
        paths.append(Path(*relative.parts))
    return paths


def profile_updates(kit: Path, payload: Path) -> dict[Path, bytes]:
    """Migrate personal profiles in memory; invalid input stops before any write."""
    base = kit / 'weapons.ini'
    originals = [base, *sorted((kit / 'presets').glob('*.ini'))]
    updates = {}
    for path in originals:
        raw = path.read_bytes() if path.is_file() else (payload / 'launcher/weapons.default.ini').read_bytes()
        try:
            new = convert(raw)[0]
            core.checked_document(new)
        except (ValueError, UnicodeError) as error:
            raise ValueError(f'Fix this configuration before updating: {path}\n{error}') from error
        updates[path] = new
    return updates


def install(payload: Path, source: Path, kit: Path, manifest: Path) -> Path:
    if source == payload or source in payload.parents or payload in source.parents:
        raise ValueError('Select your existing ReRAC source folder, separate from this update bundle.')
    if source == kit or source in kit.parents or kit in source.parents:
        raise ValueError('The source and mod launcher folders must be separate, without nesting.')
    if kit == payload or payload in kit.parents or kit in payload.parents:
        raise ValueError('Select an existing mod folder outside the delivered source tree.')
    required = ('Cargo.toml', 'Cargo.lock', 'crates/rc-game/Cargo.toml', 'crates/rc-engine/Cargo.toml')
    missing = [name for name in required if not (source/name).is_file()]
    if missing: raise ValueError('Select a ReRAC source checkout containing Cargo.toml and crates: '+', '.join(missing))
    files = source_files(payload, manifest)
    updates = profile_updates(kit, payload)
    settings = core.read_settings(kit)
    settings['exe'] = 'runtime/rerac.exe'
    updates[kit / 'launcher-settings.json'] = (json.dumps(settings, indent=2) + '\n').encode('utf-8')
    for relative in files:
        updates[source / relative] = (payload / relative).read_bytes()
        if relative.parts[0] == 'launcher' and len(relative.parts) == 2:
            # The Python test editor uses the same source and custom icon as the
            # frozen player EXE, without paying for a PyInstaller build each time.
            target = kit / relative.name
            if relative.name == 'launcher.ico' and target.is_file():
                continue
            updates[target] = (payload / relative).read_bytes()
    opener = ('@echo off\r\nsetlocal DisableDelayedExpansion\r\ncd /d "%~dp0"\r\n'
              'where py >nul 2>&1\r\nif not errorlevel 1 (\r\n'
              '  py -3 "%~dp0launcher.py" --folder "%~dp0."\r\n) else (\r\n'
              '  python "%~dp0launcher.py" --folder "%~dp0."\r\n)\r\npause\r\n')
    updates[kit / 'OPEN_TEST_LAUNCHER_v59.bat'] = opener.encode('utf-8')
    changed = {p: data for p, data in updates.items() if not p.is_file() or p.read_bytes() != data}
    backup = source / ('.weapon_evo_backup_v59_' + core.stamp())
    originals: dict[Path, bytes | None] = {}
    for path in changed:
        old = path.read_bytes() if path.is_file() else None
        originals[path] = old
        if old is not None:
            group, relative = ('source', path.relative_to(source)) if path.is_relative_to(source) else ('mod', path.relative_to(kit))
            core.atomic_write(backup / group / relative, old)
    core.atomic_write(backup / 'RESTORE.json', (json.dumps({
        'source': str(source), 'mod': str(kit), 'new_files': [str(p) for p,v in originals.items() if v is None],
        'note': 'Restore source/ and mod/ into their recorded folders. New files had no previous version.'
    }, indent=2) + '\n').encode('utf-8'))
    applied = []
    try:
        for path, data in changed.items():
            core.atomic_write(path, data)
            applied.append(path)
    except OSError:
        for path in reversed(applied):
            if originals[path] is None: path.unlink(missing_ok=True)
            else: core.atomic_write(path, originals[path])
        raise
    print(f'Updated {len(changed)} files. Backup: {backup}', flush=True)
    return backup


def launcher_folder(candidate: Path) -> Path:
    """Accept an installed editor or a source editor without requiring a game."""
    for folder in (candidate, candidate / 'launcher', candidate / 'source' / 'launcher'):
        if ((folder / 'launcher.py').is_file() and (folder / 'launcher_core.py').is_file()
                and any((folder / name).is_file() for name in ('weapons.ini', 'weapons.default.ini', 'weapons.example.ini'))):
            return folder.resolve()
    raise ValueError('Select the folder containing launcher.py and weapons.ini or weapons.default.ini. A source/launcher folder is supported; no runtime is required.')


def install_launcher(payload: Path, kit: Path, manifest: Path) -> Path:
    """Restore only Python launcher code; keep profiles, icon, runtime and saves."""
    if kit == payload or payload in kit.parents or kit in payload.parents:
        raise ValueError('Select the existing mod folder outside this update bundle.')
    kit = launcher_folder(kit)
    files = source_files(payload, manifest)
    updates = {kit / relative.name: (payload / relative).read_bytes() for relative in files
               if relative.parts[0] == 'launcher' and len(relative.parts) == 2 and relative.suffix == '.py'}
    opener = ('@echo off\r\nsetlocal DisableDelayedExpansion\r\ncd /d "%~dp0"\r\n'
              'where py >nul 2>&1\r\nif not errorlevel 1 (\r\n'
              '  py -3 "%~dp0launcher.py" --folder "%~dp0."\r\n) else (\r\n'
              '  python "%~dp0launcher.py" --folder "%~dp0."\r\n)\r\npause\r\n')
    updates[kit / 'OPEN_TEST_LAUNCHER_v58_2.bat'] = opener.encode('utf-8')
    changed = {path: data for path, data in updates.items() if not path.is_file() or path.read_bytes() != data}
    backup = kit / 'backup' / 'launcher_v58_2' / core.stamp()
    originals = {path: path.read_bytes() if path.is_file() else None for path in changed}
    for path, data in originals.items():
        if data is not None: core.atomic_write(backup / path.name, data)
    core.atomic_write(backup / 'RESTORE.json', (json.dumps({
        'mod': str(kit), 'new_files': [path.name for path,data in originals.items() if data is None],
        'note': 'Restore backed-up Python files to the mod folder; remove new files only if reverting this update.'
    }, indent=2) + '\n').encode('utf-8'))
    applied = []
    try:
        for path,data in changed.items():
            core.atomic_write(path,data); applied.append(path)
    except OSError:
        for path in reversed(applied):
            if originals[path] is None: path.unlink(missing_ok=True)
            else: core.atomic_write(path,originals[path])
        raise
    print(f'Restored launcher 0.17.2: {len(changed)} files. Backup: {backup}', flush=True)
    print('No game compilation. Runtime, INI values, presets and custom icon are preserved.', flush=True)
    return backup


def select_path(prompt: str, default: Path) -> Path:
    entered = input(f'{prompt} [{default}]: ').strip().strip('"')
    return Path(entered).expanduser().resolve() if entered else default.resolve()


def open_editor(kit: Path, verify_runtime: bool = True) -> int:
    if verify_runtime:
        identity = core.probe_exe(kit / 'runtime/rerac.exe', 58)
        if identity.get('weapon_evolution_patch') != '0.1.6.59':
            raise ValueError('This test needs the v59 runtime. Run 01_UPDATE_BUILD_AND_OPEN.bat first.')
    print('Opening the v59 Python test launcher. Use Save and launch to start the game.', flush=True)
    return subprocess.run([sys.executable, str(kit / 'launcher.py'), '--folder', str(kit)], cwd=kit).returncode


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'open', 'launcher'])
    parser.add_argument('--source', type=Path)
    parser.add_argument('--mod', type=Path)
    args = parser.parse_args()
    state_file = BUNDLE / 'local-test-settings.json'
    try:
        if os.name != 'nt': raise ValueError('Run this local test kit on Windows x64.')
        import tkinter  # Verify the author's existing Python installation before changing files.
        previous = json.loads(state_file.read_text(encoding='utf-8')) if state_file.is_file() else {}
        if args.action == 'open':
            kit = args.mod or (Path(previous['mod']) if 'mod' in previous else None)
            if kit is None: raise ValueError('Run 01_UPDATE_BUILD_AND_OPEN.bat once first.')
            return open_editor(kit.resolve(), verify_runtime=not previous.get('editor_only', False))
        if args.action == 'launcher':
            editabile.ensure_closed()
            kit = args.mod or select_path('Existing launcher folder (launcher.py; source/launcher is supported)', Path(previous.get('mod', r'C:\ReRAC-Mod')))
            kit = launcher_folder(kit.resolve())
            install_launcher(PAYLOAD, kit, BUNDLE / 'source-files.json')
            previous['mod'] = str(kit)
            previous['editor_only'] = True
            core.atomic_write(state_file, (json.dumps(previous, indent=2)+'\n').encode('utf-8'))
            return open_editor(kit, verify_runtime=False)
        if shutil.which('cargo') is None: raise ValueError('Rust/Cargo was not found. Use the x64 Visual Studio developer prompt with Rust installed.')
        editabile.ensure_closed()
        source = args.source or select_path('Existing ReRAC source folder', Path(previous.get('source', r'C:\ReRAC-Mod\rerac')))
        kit = args.mod or select_path('Existing mod launcher folder (weapons.ini and runtime)', Path(previous.get('mod', str(BUNDLE / 'test-launcher'))))
        source, kit = source.resolve(), kit.resolve()
        install(PAYLOAD, source, kit, BUNDLE / 'source-files.json')
        core.atomic_write(state_file, (json.dumps({'source': str(source), 'mod': str(kit)}, indent=2)+'\n').encode('utf-8'))
        print('Building only the editable game runtime; reusing: ' + str(source / 'target'), flush=True)
        editabile.build(source, kit)
        return open_editor(kit)
    except (OSError, ValueError, RuntimeError, ImportError, subprocess.SubprocessError) as error:
        print('ERROR: ' + str(error), file=sys.stderr, flush=True)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
