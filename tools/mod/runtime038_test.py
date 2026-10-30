"""Build bundled v120 sources and transactionally update an existing playable mod.

The old source checkout is never patched. Player INIs, presets and saves are
retained. The working GUI icon is recovered before rebuilding the editor. Both
native and GUI checks must succeed before installed files are changed.
"""
from __future__ import annotations
import argparse
import contextlib
import hashlib
import json
import os
import shutil
from pathlib import Path, PurePosixPath
import subprocess
import sys
import time
import traceback
from verified_artifacts import build_engine, checked_launcher

SOURCE = Path(__file__).resolve().parents[2]
BUNDLE = SOURCE.parent
PATCH = '0.1.6.123'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + '.runtime038.tmp')
    try:
        temporary.write_bytes(data)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def safe(root, name):
    relative = PurePosixPath(name)
    if relative.is_absolute() or '\\' in name or ':' in name or '..' in relative.parts:
        raise ValueError('Unsafe source path: ' + name)
    path = root.joinpath(*relative.parts).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError('Source path leaves the bundle: ' + name)
    return path


def verify_sources(source=SOURCE, bundle=BUNDLE):
    manifest = json.loads((bundle / 'source-files.json').read_text(encoding='utf-8'))
    if manifest['patch'] != PATCH or manifest['runtime_base'] != '0.3.8-alpha':
        raise ValueError('This is not the v120 / ReRAC 0.3.8 source kit.')
    for name, digest in manifest['files'].items():
        path = safe(source, name)
        if not path.is_file() or sha(path.read_bytes()) != digest:
            raise ValueError('Source integrity check failed: ' + name)


def validate_identity(identity):
    expected = {'name': 'rerac', 'game': 'rac1', 'version': '0.3.8',
                'weapon_evolution_patch': PATCH, 'weapon_evolution_config': 93,
                'weapon_evolution_audio_diagnostics': False}
    if any(identity.get(key) != value for key, value in expected.items()):
        raise ValueError('Expected editable Weapon Evolution v120 / ReRAC 0.3.8.')


def probe(exe):
    result = subprocess.run([str(exe), '--version-json'], check=True,
                            capture_output=True, text=True, encoding='utf-8', timeout=30)
    identity = json.loads(result.stdout)
    validate_identity(identity)
    return identity


def run_build(args, cwd):
    """Stream output directly, without a detached log reader that can block exit."""
    print(subprocess.list2cmdline([str(a) for a in args]), flush=True)
    process = subprocess.Popen(args, cwd=cwd, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, encoding='utf-8', errors='replace', bufsize=1)
    try:
        for line in process.stdout:
            print(line, end='', flush=True)
        code = process.wait()
        if code:
            raise subprocess.CalledProcessError(code, args)
    except BaseException:
        if process.poll() is None:
            process.terminate()
            process.wait()
        raise


def install_runtime(source, playable, launcher, identity, bundle=BUNDLE, executable=None, launcher_executable=None):
    """Replace the runtime, shader interface and optional checked GUI with rollback."""
    validate_identity(identity)
    executable = executable or source / 'target/release/rerac.exe'
    changes = {playable / 'runtime/rerac.exe': executable.read_bytes()}
    shaders = sorted((source / 'crates/rc-engine/assets/shaders').glob('*.wgsl'))
    if not shaders:
        raise ValueError('Bundled runtime shaders are missing.')
    for path in shaders:
        changes[playable / 'runtime/assets/shaders' / path.name] = path.read_bytes()
    changes[playable / 'runtime/build-info.json'] = (json.dumps({
        'mode': 'editable', 'version': identity['version'], 'mod_version': PATCH,
        'launcher_version': '0.29.6'}, indent=2) + '\n').encode()
    if launcher_executable is not None:
        changes[launcher]=launcher_executable.read_bytes()
        licenses=launcher_executable.parent/'licenses'
        if licenses.is_dir():
            for item in licenses.rglob('*'):
                if item.is_file() and not item.is_symlink():
                    changes[playable/'licenses'/item.relative_to(licenses)]=item.read_bytes()
    # Retire the ambiguous XP-only profile key inside the same rollback boundary.
    # Other keys/comments and native/companion save files are left untouched.
    sys.path.insert(0,str(source/'launcher'))
    from nanotech_cap_profile import migrate_health_cap
    profiles=[playable/'weapons.ini']
    profiles.extend(sorted((playable/'presets').glob('*.ini')))
    for profile in profiles:
        if profile.is_file() and not profile.is_symlink():
            original=profile.read_bytes()
            migrated=migrate_health_cap(original)
            if migrated!=original:changes[profile]=migrated
    backup = playable / 'backup/runtime_038'  / str(time.time_ns())
    backup.mkdir(parents=True)
    previous = {p: p.read_bytes() if p.is_file() else None for p in changes}
    records = []
    for index, (path, data) in enumerate(previous.items()):
        name = f'{index:04}.bin'
        if data is not None:
            (backup / name).write_bytes(data)
        records.append({'path': path.relative_to(playable).as_posix(),
                        'backup': name if data is not None else None})
    (backup / 'restore.json').write_text(json.dumps(records, indent=2), encoding='utf-8')
    installed = []
    try:
        for path, data in changes.items():
            write(path, data)
            installed.append(path)
        record = {'playable': str(playable), 'launcher': str(launcher), 'patch': PATCH,
                  'runtime_base': '0.3.8-alpha', 'backup': str(backup),
                  'exe_sha256': sha(changes[playable / 'runtime/rerac.exe']),
                  'launcher_sha256':sha(changes.get(launcher,launcher.read_bytes())),
                  'shaders': {p.name: sha(p.read_bytes()) for p in shaders}}
        write(bundle / 'installed.json', (json.dumps(record, indent=2) + '\n').encode())
    except BaseException:
        failures = []
        for path in reversed(installed):
            try:
                if previous[path] is None:
                    path.unlink(missing_ok=True)
                else:
                    write(path, previous[path])
            except OSError as error:
                failures.append(str(error))
        if failures:
            print('Rollback needs manual attention; backup: ' + str(backup))
            print('\n'.join(failures))
        raise
    return backup


def installed():
    record = json.loads((BUNDLE / 'installed.json').read_text(encoding='utf-8'))
    playable, launcher = Path(record['playable']), Path(record['launcher'])
    exe = playable / 'runtime/rerac.exe'
    probe(exe)
    if record.get('launcher_sha256') and sha(launcher.read_bytes())!=record['launcher_sha256']:
        raise ValueError('The installed launcher changed; build again before packaging.')
    if sha(exe.read_bytes()) != record['exe_sha256']:
        raise ValueError('The installed runtime changed; build again before packaging.')
    for name, digest in record['shaders'].items():
        if sha((playable / 'runtime/assets/shaders' / name).read_bytes()) != digest:
            raise ValueError('The installed shader changed: ' + name)
    return playable, launcher


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'open', 'package', 'health'])
    parser.add_argument('--playable', type=Path)
    parser.add_argument('--launcher', default='ReRAC_Weapon_Evolution.exe')
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--hp',type=int,default=210)
    parser.add_argument('--data-dir',type=Path)
    args = parser.parse_args()
    if os.name != 'nt':
        raise ValueError('Run this local test kit on Windows.')
    if args.action != 'build':
        playable, launcher = installed()
        if args.action == 'health':
            from nanotech_test import launch
            launch(playable,BUNDLE/'logs'/('nanotech-'+str(time.time_ns())),args.hp,args.data_dir)
            return
        if args.action == 'open':
            os.startfile(launcher)
            return
        sys.path.insert(0, str(SOURCE / 'launcher'))
        from launcher_core import package_distribution
        output = BUNDLE / 'dist' / ('ReRAC_Weapon_Evolution_0.4_ReRAC_0.3.8_Test_' + str(time.time_ns()))
        archive = package_distribution(SOURCE / 'launcher', output,
            (playable / 'weapons.ini').read_bytes(), playable / 'runtime/rerac.exe',
            include_presets=False, launcher_exe=launcher)
        print('Playable ZIP: ' + str(archive))
        return
    verify_sources()
    print('Weapon Evolution v120 / ReRAC 0.3.8-alpha. Orxon shortcut fix 118. No GitHub operations.')
    playable = args.playable or Path(input('EXISTING playable mod folder (launcher EXE, weapons.ini and runtime): ').strip().strip('"'))
    playable = playable.expanduser().resolve()
    launcher = (playable / args.launcher).resolve()
    if launcher.parent != playable or not launcher.is_file() or not (playable / 'weapons.ini').is_file() or not (playable / 'runtime/rerac.exe').is_file():
        raise ValueError('Select the existing playable mod folder, not this kit or source.')
    tasks = subprocess.run(['tasklist', '/fi', 'IMAGENAME eq rerac.exe', '/fo', 'csv', '/nh'],
        check=True, capture_output=True, text=True, errors='replace')
    if '"rerac.exe"' in tasks.stdout.lower():
        raise ValueError('Close the game before updating its runtime.')
    gui_tasks=subprocess.run(['tasklist','/fi','IMAGENAME eq '+launcher.name,'/fo','csv','/nh'],
        check=True,capture_output=True,text=True,errors='replace')
    if ('"'+launcher.name.lower()+'"') in gui_tasks.stdout.lower():
        raise ValueError('Close the mod launcher before updating its EXE.')
    from windows_build_env import ensure_build_environment
    ensure_build_environment()
    from verified_artifacts import validate_source_apis
    from rust_build_tools import ensure_rust_compiler
    validate_source_apis(SOURCE)
    ensure_rust_compiler(SOURCE,BUNDLE)
    caches = [BUNDLE.parent / 'rerac/target', Path('C:/ReRAC-Mod/rerac/target')]
    target = (args.target_dir or next((p for p in caches if p.is_dir()), SOURCE / 'target')).resolve()
    print('Compiling bundled source: ' + str(SOURCE))
    print('Cargo cache: ' + str(target))
    common = ['--release', '--locked', '--no-default-features', '--target-dir', str(target)]
    # v120.2: build_engine rebuilds and tests the local workspace.
    exe = build_engine(SOURCE, target, validate_identity)
    # Build from a private stage: preserve the actual installed EXE icon without
    # mutating reviewed sources, INIs or the working GUI before all checks pass.
    sys.path.insert(0,str(SOURCE/'launcher'))
    import build_launcher_exe
    from icon_assets import extract_ico
    stage=BUNDLE/'launcher-build-input'
    stage.mkdir(exist_ok=True)
    for item in (SOURCE/'launcher').iterdir():
        if item.is_file() and item.suffix in {'.py','.ini','.ico'}:
            shutil.copyfile(item,stage/item.name)
    try:
        (stage/'launcher.ico').write_bytes(extract_ico(launcher))
    except ValueError as error:
        print('Installed EXE icon could not be recovered; using bundled icon: '+str(error))
    gui = checked_launcher(stage, build_launcher_exe)
    if gui is None:
        gui = build_launcher_exe.build(stage)
    backup = install_runtime(SOURCE, playable, launcher, probe(exe), executable=exe,launcher_executable=gui)
    print('Runtime, matching shaders and launcher 0.29.6 updated. INI, presets and saves retained.')
    print('Backup: ' + str(backup))
    print('Open this same launcher: ' + str(launcher))
    try:
        os.startfile(launcher)
    except OSError as error:
        print('Open the launcher manually: ' + str(error))


class Tee:
    def __init__(self, console, log):
        self.console, self.log = console, log

    def write(self, value):
        self.console.write(value)
        self.log.write(value)
        self.flush()
        return len(value)

    def flush(self):
        self.console.flush()
        self.log.flush()


if __name__ == '__main__':
    logs = BUNDLE / 'logs'
    logs.mkdir(exist_ok=True)
    with (logs / ('runtime038-' + str(time.time_ns()) + '.log')).open('w', encoding='utf-8') as log:
        with contextlib.redirect_stdout(Tee(sys.stdout, log)), contextlib.redirect_stderr(Tee(sys.stderr, log)):
            try:
                main()
            except Exception as error:
                print('ERROR: ' + str(error))
                (logs / 'update-error.log').write_text(traceback.format_exc(), encoding='utf-8')
                print('Diagnostic: ' + str(logs / 'update-error.log'))
                sys.exit(1)
