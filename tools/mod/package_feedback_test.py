"""Package the installed test build; never upload or publish a release."""
from pathlib import Path
import json,subprocess,sys
HERE=Path(__file__).resolve().parent

def main():
    saved=json.loads((HERE/'installed.json').read_text()) if (HERE/'installed.json').is_file() else {}
    default=saved.get('playable','')
    playable=Path(input('PLAYABLE mod folder'+(' ['+default+']' if default else '')+': ').strip().strip('"') or default).resolve()
    launcher=Path(saved.get('launcher','')) if default and playable==Path(default) else playable
    if not (launcher/'package_launcher.py').is_file():
        launcher=Path(input('Launcher SOURCE folder containing package_launcher.py: ').strip().strip('"')).resolve()
    exe=Path(saved.get('exe','')) if default and playable==Path(default) else playable/'ReRAC_Weapon_Evolution.exe'
    if not exe.is_file():exe=Path(input('Full path of the installed launcher EXE: ').strip().strip('"')).resolve()
    if not (playable/'runtime/rerac.exe').is_file() or not (playable/'weapons.ini').is_file():raise ValueError('Select the playable mod folder containing runtime and weapons.ini.')
    import datetime
    name='ReRAC_Weapon_Evolution_0.3_v102_'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
    output=playable/'dist'/name
    args=[sys.executable,str(launcher/'package_launcher.py'),'--exe',str(playable/'runtime/rerac.exe'),
        '--ini',str(playable/'weapons.ini'),'--launcher-exe',str(exe),'--output',str(output),'--no-presets']
    print(subprocess.list2cmdline(args),flush=True);subprocess.run(args,cwd=launcher,check=True)
    print('Local package created under '+str(playable/'dist')+'. Nothing was uploaded or published.')
if __name__=='__main__':
    try:main()
    except Exception as error:print('ERROR: '+str(error));sys.exit(1)
