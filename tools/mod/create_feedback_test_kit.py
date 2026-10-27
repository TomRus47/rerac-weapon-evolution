"""Create a hash-checked local v102 delta from reviewed source/launcher baselines."""
from pathlib import Path
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[2]
def digest(data):return hashlib.sha256(data.replace(b'\r\n',b'\n')).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline-source',type=Path,required=True);p.add_argument('--baseline-launcher',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    rows=[];payload={}
    sources=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',*[f for f in (ROOT/'crates').rglob('*') if f.is_file() and f.suffix in {'.rs','.toml','.wgsl'}]]
    launchers=[f for f in (ROOT/'launcher').glob('*.py') if not f.name.startswith('test_')]
    for kind,files,base,oldbase in [('source',sources,ROOT,a.baseline_source),('launcher',launchers,ROOT/'launcher',a.baseline_launcher)]:
        for f in files:
            name=f.relative_to(base).as_posix();old=oldbase/name
            before=old.read_bytes() if old.is_file() else None;data=f.read_bytes()
            if before is not None and digest(before)==digest(data) and not (kind=='launcher' and name in {'launcher.py','build_launcher_exe.py'}):continue
            rows.append({'root':kind,'path':name,'before':[digest(before) if before is not None else None],'after':digest(data)})
            payload['payload/'+kind+'/'+name]=data
    prefix='ReRAC_Weapon_Evolution_v102_Feedback_HUD/'
    a.output.parent.mkdir(parents=True,exist_ok=True)
    if a.output.exists():raise ValueError('Choose a new output ZIP path.')
    with zipfile.ZipFile(a.output,'w',zipfile.ZIP_DEFLATED,compresslevel=9)as z:
        for name,data in payload.items():z.writestr(prefix+name,data)
        z.writestr(prefix+'manifest.json',json.dumps({'patch':'0.1.6.102','files':rows},indent=2)+'\n')
        for name in ['feedback_update.py','package_feedback_test.py','stage_feedback_profiles.py','windows_build_env.py']:
            z.writestr(prefix+name,(ROOT/'tools/mod'/name).read_bytes())
        for bat,script in [('01_UPDATE_BUILD_AND_OPEN.bat','feedback_update.py'),('02_CREATE_PLAYABLE_ZIP.bat','package_feedback_test.py')]:
            text='@echo off\nsetlocal DisableDelayedExpansion\ncd /d "%~dp0"\npy -3 '+script+'\nset "RESULT=%ERRORLEVEL%"\necho.\necho Operation exited with code %RESULT%.\npause\nexit /b %RESULT%\n'
            z.writestr(prefix+bat,text.replace('\n','\r\n'))
        z.writestr(prefix+'README_FIRST.md',(ROOT/'docs/mod/FEEDBACK_HUD_V102.md').read_bytes())
        z.writestr(prefix+'LICENSE',(ROOT/'LICENSE').read_bytes())
    with zipfile.ZipFile(a.output)as z:
        if z.testzip() is not None:raise ValueError('ZIP integrity check failed.')
    print(str(a.output.resolve())+'\n'+str(len(rows))+' reviewed source/launcher files; '+str(a.output.stat().st_size)+' bytes')
if __name__=='__main__':main()
