"""Package the exact-block v105-to-v106 Windows runtime update."""
from pathlib import Path
import argparse, hashlib, json, zipfile
ROOT=Path(__file__).resolve().parents[2]
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.output.exists():raise ValueError('Choose a new output ZIP path.')
    manifest=json.loads((ROOT/'tools/mod/v106_slam_manifest.json').read_text())
    contents={
        'manifest.json':(ROOT/'tools/mod/v106_slam_manifest.json').read_bytes(),
        'update_v106.py':(ROOT/'tools/mod/slam_visual_update.py').read_bytes(),
        'package_v106.py':(ROOT/'tools/mod/package_slam_visual.py').read_bytes(),
        'windows_build_env.py':(ROOT/'tools/mod/windows_build_env.py').read_bytes(),
        'README_FIRST.md':(ROOT/'docs/mod/SLAM_VISUAL_V106.md').read_bytes(),
        'LICENSE':(ROOT/'LICENSE').read_bytes(),
    }
    for row in manifest['files']:
        body=(ROOT/row['path']).read_bytes()
        if hashlib.sha256(body.replace(b'\r\n',b'\n')).hexdigest()!=row['sha256']:
            raise ValueError('Payload differs from reviewed source: '+row['path'])
        contents['payload/'+row['file']]=body
    for name,script in [('01_UPDATE_BUILD_AND_OPEN.bat','update_v106.py'),('02_CREATE_PLAYABLE_ZIP.bat','package_v106.py')]:
        text='@echo off\nsetlocal DisableDelayedExpansion\ncd /d "%~dp0"\npy -3 '+script+'\nset "RESULT=%ERRORLEVEL%"\necho.\necho Operation exited with code %RESULT%.\npause\nexit /b %RESULT%\n'
        contents[name]=text.replace('\n','\r\n').encode()
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(args.output,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for name,body in contents.items():z.writestr('ReRAC_Weapon_Evolution_v106_Blast_Size/'+name,body)
    with zipfile.ZipFile(args.output) as z:
        if z.testzip():raise ValueError('ZIP integrity check failed.')
    print(args.output.resolve())
if __name__=='__main__':main()
