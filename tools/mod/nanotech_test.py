"""Launch a disposable Veldin II health test using the installed v116 runtime."""
from __future__ import annotations
import argparse
from pathlib import Path
import os
import subprocess
import sys
import time

SOURCE = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(SOURCE / 'launcher'))
import launcher_core as core
from migrate_ini import runtime_profile


def test_environment(ini, data, hp, env=None):
    result=core.game_environment(ini,data,env=env)
    # A test cannot inherit a different direct-boot scenario from the shell.
    for key in ('RC_WEAPON_LOCATION_TEST','RC_GEMLIK_GOLD_TEST','RC_WEAPON_EVO_TEST',
                'RC_PLAY_SCRIPT','RC_PLAY_FLY','RC_GIVE_ITEMS','RC_GIVE_BOLTS'):
        result.pop(key,None)
    result.update(RC_NANOTECH_TEST='1',RC_NANOTECH_TEST_HP=str(max(8,min(210,hp))),
                  RC_SAVE_DIR='0',RC_LEVEL='18',RC_FRONTEND='0',RC_SHIP='2')
    return result


def find_data(playable,exe,explicit=None):
    settings=core.read_settings(playable)
    saved=core.resolve_path(settings.get('data'),playable)
    data=explicit or (saved if core.valid_data(saved) else core.discover_data(playable,exe))
    if not core.valid_data(data):
        data=Path(input('EXTRACTED game data folder containing toc.bin: ').strip().strip('"')).expanduser().resolve()
    if not core.valid_data(data):
        raise ValueError('Choose the extracted game data folder containing toc.bin and extract-info.json.')
    return data


def launch(playable,output,hp=210,data=None):
    exe=playable/'runtime/rerac.exe'
    core.probe_exe(exe,92)
    core.ensure_closed()
    data=find_data(playable,exe,data)
    output.mkdir(parents=True,exist_ok=False)
    ini=output/'weapons-test.ini'
    ini.write_bytes(runtime_profile((playable/'weapons.ini').read_bytes()))
    env=test_environment(ini,data,hp)
    print('Veldin II health test. Normal saves are disabled. Initial HP: '+env['RC_NANOTECH_TEST_HP'],flush=True)
    print('F5: next health boundary. F6: full heal. F7: remove one HP (minimum one).',flush=True)
    print('Use enemies to test ordinary damage and death. Test log: '+str(output/'game.log'),flush=True)
    with (output/'game.log').open('w',encoding='utf-8') as log:
        result=subprocess.run([str(exe),'--data-dir',str(data)],cwd=exe.parent,
                              env=env,stdout=log,stderr=subprocess.STDOUT)
    if result.returncode:
        raise ValueError('Test runtime exited with code '+str(result.returncode)+'. Read '+str(output/'game.log'))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--playable',type=Path,required=True)
    parser.add_argument('--data-dir',type=Path)
    parser.add_argument('--hp',type=int,default=210)
    args=parser.parse_args()
    launch(args.playable.resolve(),SOURCE.parent/'logs'/('nanotech-'+str(time.time_ns())),args.hp,args.data_dir)

if __name__=='__main__':
    try:main()
    except Exception as error:
        print('ERROR: '+str(error),flush=True)
        raise SystemExit(1)
