"""Create a standalone local player ZIP from the verified existing installation."""
from pathlib import Path
import datetime,hashlib,json,shutil,subprocess,sys,zipfile
HERE=Path(__file__).resolve().parent
def main():
    info=json.loads((HERE/'installed.json').read_text());root=Path(info['playable']);launcher=Path(info['launcher_exe'])
    version=json.loads(subprocess.run([str(root/'runtime/rerac.exe'),'--version-json'],capture_output=True,text=True,check=True).stdout)
    if version.get('weapon_evolution_patch')!='0.1.6.106':raise ValueError('Build and install v106 before packaging.')
    name='ReRAC_Weapon_Evolution_0.3_v106_'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
    out=root/'dist'/name;out.mkdir(parents=True)
    shutil.copytree(root/'runtime',out/'runtime');shutil.copy2(launcher,out/launcher.name);shutil.copy2(root/'weapons.ini',out/'weapons.ini')
    for name in ['launcher.ico','LICENSE','README_FIRST.md']:
        path=root/name if (root/name).is_file() else HERE/name
        if path.is_file():shutil.copy2(path,out/name)
    if (root/'licenses').is_dir():shutil.copytree(root/'licenses',out/'licenses')
    (out/'PLAY_FIRST.txt').write_text('Extract the complete package and open '+launcher.name+'. Keep runtime and weapons.ini together. Select your own extracted game-data folder if requested. No original game assets are included. Weapon Evolution is an independent mod by TomRus47; the author does not develop ReRAC or the original games and owns no original-game rights or assets. Upstream licenses and credits remain applicable.\n',encoding='utf-8')
    archive=out.with_suffix('.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for f in out.rglob('*'):
            if f.is_file():z.write(f,f.relative_to(out.parent))
    archive.with_suffix('.zip.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n')
    print('Local player ZIP: '+str(archive)+'\nNo upload or publication was performed.')
if __name__=='__main__':
    try:main()
    except Exception as e:print('ERROR: '+str(e));sys.exit(1)
