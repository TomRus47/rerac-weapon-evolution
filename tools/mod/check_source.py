#!/usr/bin/env python3
"""Check the tracked source export without needing Rust or extracted game data."""
from pathlib import Path
import hashlib,json,re,subprocess,sys

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'launcher'))
import launcher_core as core
from migrate_ini import runtime_profile

def check():
    listing=subprocess.run(['git','ls-files','-z'],cwd=ROOT,check=True,capture_output=True).stdout
    files=[name.decode('utf-8') for name in listing.split(b'\0') if name]
    if not files:raise ValueError('No tracked source files. Initialize and stage the source checkout first.')
    forbidden={'.exe','.dll','.pdb','.iso','.zip','.bundle','.pyc'}
    roots={'.cargo','.gitattributes','.gitignore','01_BUILD_GAME.bat','02_OPEN_LAUNCHER.bat',
        'BUILD_FINAL_WINDOWS_EXE.bat','Cargo.lock','Cargo.toml','LICENSE','CHANGELOG.md',
        'README.md','README.ReRAC.md','MOD_FILES.md','crates','tools','docs','launcher','mod'}
    for name in files:
        path=ROOT/name
        if Path(name).parts[0] not in roots:raise ValueError('Unexpected source root entry: '+name)
        if path.name in {'toc.bin','extract-info.json'}:raise ValueError('Game data marker in source export: '+name)
        if path.suffix.lower() in forbidden or '.prima_' in name or '.weapon_evo_backup' in name:
            raise ValueError('Unexpected generated/private file: '+name)
        if path.stat().st_size>10*1024*1024:raise ValueError('Unexpected large source file: '+name)
        if path.suffix in {'.rs','.py','.md','.ini','.json','.toml','.bat'}:
            text=path.read_text(encoding='utf-8')
            if re.search(r'C:[\\/]Users[\\/](?!USERNAME)[^\\/\s]+',text,re.I):
                raise ValueError('Personal Windows path: '+name)
            if '/workspace/'+'scratch/' in text or '/Users/'+'aslanhud/' in text:
                raise ValueError('Personal preparation path: '+name)
    default=(ROOT/'launcher/weapons.default.ini').read_bytes()
    if default!=(ROOT/'launcher/weapons.example.ini').read_bytes():raise ValueError('Public default/example differ.')
    doc=core.profile_document(default)
    if ('general','damage_mode') in doc.entries:raise ValueError('Public default exposes the private damage marker.')
    embedded=ROOT/'crates/rc-game/src/weapon_evo_release.ini'
    if embedded.read_bytes()!=runtime_profile(default):raise ValueError('Optional embedded profile differs from the public default.')
    record=json.loads((ROOT/'mod/source-map.json').read_text())
    for item in record['files']:
        actual=hashlib.sha256((ROOT/item['path']).read_bytes()).hexdigest()
        if actual!=item['sha256']:raise ValueError('Export source hash differs: '+item['path'])
    for name in ['rc-game','rc-engine','rc-formats','rc-data','rc-video','rc-extract']:
        if not (ROOT/'crates'/name/'Cargo.toml').is_file():raise ValueError('Missing workspace crate: '+name)
    cap=(ROOT/'crates/rc-engine/src/disc_source.rs').read_text()
    if '"weapon_evolution_config":40' not in cap:raise ValueError('Missing v40 runtime capability.')
    local=['target/release/rerac.exe','extracted/toc.bin','launcher/runtime/rerac.exe',
        'launcher/launcher-settings.json','launcher/weapons.ini','launcher/presets/Personal.ini',
        'launcher/runs/session/weapons.ini','launcher/.launcher-tools/venv/python.exe',
        'launcher/dist/Player.zip','launcher/launcher.ico']
    ignored=subprocess.run(['git','check-ignore','--stdin'],cwd=ROOT,input='\n'.join(local)+'\n',capture_output=True,text=True)
    if set(ignored.stdout.splitlines())!=set(local):raise ValueError('Local output is not completely ignored by Git.')
    print(f'Source export OK: {len(files)} tracked files, {len(record["files"])} indexed mod files, {len(doc.entries)} default INI values.')

if __name__=='__main__':
    try:check()
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        print('ERROR: '+str(error),file=sys.stderr);raise SystemExit(1)
