#!/usr/bin/env python3
"""Build once without the fixed-profile feature; relaunch with this folder's INI."""
from __future__ import annotations
import argparse, hashlib, json, os, re, shutil, subprocess, sys, tempfile
from pathlib import Path
from launcher_core import atomic_write
from valida_ini import validate

KIT = Path(__file__).resolve().parent
DEFAULT_ROOT = KIT.parent

def prepare_local_ini(kit: Path) -> Path:
    path = kit/'weapons.ini'
    if not path.exists():
        source = kit/'weapons.example.ini'
        if not source.is_file(): raise ValueError('Missing weapons.ini. Place your INI beside the BAT files.')
        atomic_write(path,source.read_bytes())
        print('INI copied from: '+str(source),flush=True)
        if source.name == 'weapons.example.ini':
            print('Using example defaults because no previous INI was found.',flush=True)
    raw = path.read_bytes()
    from migrate_ini import convert
    converted, _ = convert(raw)
    lines = converted.decode('utf-8-sig').splitlines(keepends=True)
    section = ''
    kept = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('[') and stripped.endswith(']'): section = stripped[1:-1].strip().lower()
        if section == 'hud' and re.match(r'\s*xp_bar_opacity_percent\s*=',line,re.I): continue
        kept.append(line)
    prefix = b'\xef\xbb\xbf' if converted.startswith(b'\xef\xbb\xbf') else b''
    updated = prefix + ''.join(kept).encode('utf-8')
    if updated != raw:
        backup = kit/'weapons.ini.before-v102.bak'
        if not backup.exists():
            with backup.open('xb') as stream: stream.write(raw)
        atomic_write(path,updated)
        print('INI normalized; the original was backed up.',flush=True)
    return path

def check_source_root(root: Path) -> None:
    required = ['Cargo.toml', 'Cargo.lock', 'crates/rc-game/Cargo.toml',
                'crates/rc-engine/Cargo.toml', 'crates/rc-game/src/moby_update/real_hp.rs']
    missing = [name for name in required if not (root/name).is_file()]
    if missing: raise ValueError('Incomplete source checkout: '+', '.join(missing))

def editable_version(exe: Path) -> dict:
    result = subprocess.run([str(exe),'--version-json'],check=True,capture_output=True,
                            text=True,encoding='utf-8',timeout=20)
    version = json.loads(result.stdout.strip())
    if (version.get('name') != 'rerac' or version.get('game') != 'rac1'
            or version.get('data_format') != 1 or not version.get('version')):
        raise ValueError('Unrecognized ReRAC executable.')
    if str(version['version']).endswith('-fixed'):
        raise ValueError('This is a fixed-profile build. Rebuild with 01_RICOMPILA_E_PROVA.bat.')
    if version.get('weapon_evolution_config',0) < 90:
        raise ValueError('This executable lacks configuration capability 90. Rebuild the current game.')
    return version

def ensure_closed() -> None:
    if os.name == 'nt':
        result = subprocess.run(['tasklist','/FI','IMAGENAME eq rerac.exe','/FO','CSV','/NH'],
                                capture_output=True,text=True,errors='replace',check=True)
        if '"rerac.exe"' in result.stdout.lower():
            raise ValueError('Close ReRAC before continuing.')

def stage_runtime(root: Path, kit: Path, exe: Path) -> Path:
    version = editable_version(exe)
    shaders = sorted((root/'crates/rc-engine/assets/shaders').glob('*.wgsl'))
    if not shaders:
        raise ValueError('Project shaders are missing.')
    stage = Path(tempfile.mkdtemp(prefix='.runtime_stage_',dir=kit))
    try:
        shutil.copy2(exe,stage/'rerac.exe')
        (stage/'assets/shaders').mkdir(parents=True)
        for shader in shaders:
            shutil.copy2(shader,stage/'assets/shaders'/shader.name)
        (stage/'build-info.json').write_text(json.dumps({'mode':'editable','root':str(root.resolve()),
            'version':version['version'],'mod_version':version.get('weapon_evolution_patch','unknown')},indent=2)+'\n',encoding='utf-8')
        runtime = kit/'runtime'
        previous = kit/'runtime_previous'
        if previous.exists():
            shutil.rmtree(previous)
        if runtime.exists():
            runtime.rename(previous)
        try:
            stage.rename(runtime)
        except OSError:
            if previous.exists(): previous.rename(runtime)
            raise
        if previous.exists(): shutil.rmtree(previous)
        return runtime
    finally:
        if stage.exists(): shutil.rmtree(stage)

def build(root: Path, kit: Path) -> Path:
    if os.name != 'nt':
        raise ValueError('Run the build BAT on Windows with Rust and Visual Studio.')
    if shutil.which('cargo') is None:
        raise ValueError('Cargo was not found. Use x64 Native Tools Command Prompt for VS with Rust installed.')
    prepare_local_ini(kit)
    validate(kit/'weapons.ini')
    ensure_closed()
    check_source_root(root)
    print('Building editable Weapon Evolution from this source checkout.',flush=True)
    # An explicit target dir also prevents a user-wide CARGO_TARGET_DIR from picking a stale executable.
    common = ['--release','--locked','--no-default-features','--target-dir',str(root/'target')]
    for args in [['cargo','test',*common,'-p','rc-game','--lib'],
                 ['cargo','build',*common,'-p','rc-engine','--bin','rerac']]:
        print(subprocess.list2cmdline(args),flush=True)
        subprocess.run(args,cwd=root,check=True)
    return stage_runtime(root,kit,root/'target/release/rerac.exe')

def runtime_environment(kit: Path, env: dict) -> dict:
    result = dict(env)
    for key in ['RC_LEVEL','RC_GIVE_ITEMS','RC_GIVE_GOLD_WEAPONS','RC_WEAPON_EVO_TEST',
                'RC_WEAPON_EVO_ICON_TEST_LEVEL','RC_DEBUG_HIT','RC_PLAY_SCRIPT','RC_PLAY_FLY']:
        result.pop(key,None)
    if result.get('RC_SAVE_DIR') == '0': result.pop('RC_SAVE_DIR')
    result.update(RC_WEAPON_EVOLUTION_INI=str((kit/'weapons.ini').resolve()),
                  RC_COMBAT_STRAFE='1',RC_BLASTER_SHOT_GATE_TICKS='2',RC_FRONTEND='1')
    return result

def data_directory(root: Path, env: dict) -> Path:
    explicit = env.get('RC_DATA_DIR') or env.get('RC_EXTRACTED')
    if explicit:
        candidates = [Path(explicit)]
    else:
        candidates = []
        if env.get('LOCALAPPDATA'):
            candidates.append(Path(env['LOCALAPPDATA'])/'rerac/games/rac1/data')
        candidates.append(root/'extracted')
    for path in candidates:
        if (path/'toc.bin').is_file() and (path/'extract-info.json').is_file():
            return path.resolve()
    raise ValueError('Extracted game data was not found. Set RC_DATA_DIR to the folder containing toc.bin and extract-info.json.')

def launch(kit: Path, root_override: Path | None = None) -> int:
    prepare_local_ini(kit)
    validate(kit/'weapons.ini')
    runtime = kit/'runtime'
    if not (runtime/'rerac.exe').is_file() or not (runtime/'build-info.json').is_file():
        raise ValueError('First run 01_RICOMPILA_E_PROVA.bat once.')
    editable_version(runtime/'rerac.exe')
    info = json.loads((runtime/'build-info.json').read_text(encoding='utf-8'))
    if info.get('mode') != 'editable': raise ValueError('This build does not support editable configurations.')
    root = root_override or Path(info['root'])
    env = runtime_environment(kit,os.environ)
    from migrate_ini import runtime_profile
    from launcher_core import stamp
    session_ini=kit/'runs'/('manual-'+stamp())/'weapons.ini'
    session_ini.parent.mkdir(parents=True)
    atomic_write(session_ini,runtime_profile((kit/'weapons.ini').read_bytes()))
    env['RC_WEAPON_EVOLUTION_INI']=str(session_ini.resolve())
    data = data_directory(root,env)
    ensure_closed()
    log = kit/'avvio_editabile.log'
    print('ACTIVE INI: '+env['RC_WEAPON_EVOLUTION_INI'],flush=True)
    print('Log: '+str(log)+'\nClose the game before editing the INI.',flush=True)
    with log.open('w',encoding='utf-8') as stream:
        result = subprocess.run([str(runtime/'rerac.exe'),'--data-dir',str(data)],
                                cwd=runtime,env=env,stdout=stream,stderr=subprocess.STDOUT)
    # Read only after process exit; no background log-following task blocks completion.
    loaded = False
    with log.open(encoding='utf-8',errors='replace') as stream:
        for line in stream:
            if 'weapon evolution: config loaded from ' in line:
                print(line.rstrip(),flush=True)
                loaded = True
    if not loaded:
        raise ValueError('The game did not confirm loading the INI. Check '+str(log))
    if result.returncode:
        print(f'Game exited with code {result.returncode}. Log: {log}',file=sys.stderr)
    return result.returncode

def main() -> int:
    parser = argparse.ArgumentParser(description='Weapon Evolution editable source build')
    parser.add_argument('action',choices=['build','run','build-and-run'])
    parser.add_argument('root',nargs='?',type=Path)
    args = parser.parse_args()
    try:
        if args.action in {'build','build-and-run'}: build((args.root or DEFAULT_ROOT).resolve(),KIT)
        if args.action in {'run','build-and-run'}: return launch(KIT,args.root)
        return 0
    except (OSError,ValueError,RuntimeError,KeyError,subprocess.SubprocessError) as error:
        print('ERROR: '+str(error),file=sys.stderr)
        return 2

if __name__ == '__main__': raise SystemExit(main())
