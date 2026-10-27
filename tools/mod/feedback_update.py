"""Install the reviewed v102 source delta, compile, and replace the selected EXEs.

The local kit has payload/ and manifest.json beside this script. Profile values,
presets, icon and saves belong to the existing installation and are preserved.
"""
from pathlib import Path,PurePosixPath
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,sys,tempfile,time
from windows_build_env import ensure_build_environment
HERE=Path(__file__).resolve().parent
PATCH='0.1.6.102'

def digest(data):return hashlib.sha256(data.replace(b'\r\n',b'\n')).hexdigest()
def write(path,data):
    path.parent.mkdir(parents=True,exist_ok=True);temporary=path.with_name(path.name+'.v102.tmp')
    try:temporary.write_bytes(data);os.replace(temporary,path)
    finally:temporary.unlink(missing_ok=True)

def plan(source,launcher,bundle=HERE):
    changes={};manifest=json.loads((bundle/'manifest.json').read_text())
    if manifest.get('patch')!=PATCH:raise ValueError('Extract the complete v102 update kit.')
    for row in manifest['files']:
        relative=PurePosixPath(row['path'])
        if relative.is_absolute() or '..' in relative.parts or '\\' in row['path'] or ':' in row['path']:raise ValueError('Unsafe update path.')
        base={'source':source,'launcher':launcher}.get(row['root'])
        if base is None:raise ValueError('Unknown payload root.')
        target=base.joinpath(*relative.parts);payload=bundle/'payload'/row['root']/relative
        if target.is_symlink() or not target.resolve().is_relative_to(base.resolve()):raise ValueError('Update path escapes the selected folder.')
        new=payload.read_bytes()
        if digest(new)!=row['after']:raise ValueError('Damaged payload: '+row['path'])
        old=target.read_bytes() if target.is_file() else None
        actual=digest(old) if old is not None else None
        if actual not in [*row['before'],row['after']]:
            raise ValueError('Unrecognized local '+row['root']+' file: '+row['path']+'. No files changed. Select the current source folders supplied for this update.')
        if old!=new:changes[target]=new
    return changes

def verify_launcher(exe,run=subprocess.run):
    with tempfile.TemporaryDirectory(prefix='v102_gui_') as td:
        report=Path(td)/'self-test.json'
        run([str(exe.resolve()),'--self-test',str(report)],check=True,timeout=120)
        data=json.loads(report.read_text())
    if not data.get('frozen') or data.get('launcher_version')!='0.28.7' or not data.get('feedback_controls_ui') or data.get('moving_wrench_gui_grades')!=8:
        raise ValueError('The built EXE does not verify the v102 grade controls. The old launcher is retained.')

def install(source,playable,launcher,target,bundle=HERE,run=subprocess.run,verify=verify_launcher):
    changes=plan(source,launcher,bundle)
    if (playable/'launcher.ico').is_file():changes[launcher/'launcher.ico']=(playable/'launcher.ico').read_bytes()
    elif (launcher/'icon_assets.py').is_file():
        spec=importlib.util.spec_from_file_location('v102_icon_preserve',launcher/'icon_assets.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        icon=module.extract_ico(target)
        if icon:changes[launcher/'launcher.ico']=icon
    backup=playable/('backup_v102_'+str(time.time_ns()));backup.mkdir()
    originals={};records=[]
    def remember(path):
        if path in originals:return
        old=path.read_bytes() if path.is_file() else None;originals[path]=old
        name=str(len(records))+'-'+path.name
        if old is not None:(backup/name).write_bytes(old)
        records.append({'path':str(path),'backup':name if old is not None else None})
        (backup/'RESTORE.json').write_text(json.dumps(records,indent=2))
    for path in [*changes,target]:remember(path)
    staged=playable/('.runtime_v102_'+str(time.time_ns()));previous=backup/'runtime';switched=False
    try:
        for path,data in changes.items():write(path,data)
        profiles=backup/'profiles'
        run([sys.executable,str(bundle/'stage_feedback_profiles.py'),str(launcher),str(playable),str(source),str(profiles)],check=True)
        for row in json.loads((profiles/'profiles.json').read_text()):
            path=Path(row['target']);remember(path);write(path,Path(row['staged']).read_bytes())
        common=['--release','--locked','--no-default-features','--target-dir',str(source/'target')]
        for args in [['cargo','test',*common,'-p','rc-game','--lib'],['cargo','build',*common,'-p','rc-engine','--bin','rerac']]:
            print(subprocess.list2cmdline(args),flush=True);run(args,cwd=source,check=True)
        game=source/'target/release/rerac.exe'
        version=json.loads(run([str(game),'--version-json'],capture_output=True,text=True,check=True,timeout=30).stdout)
        if version.get('weapon_evolution_patch')!=PATCH or version.get('weapon_evolution_config',0)<88 or str(version.get('version','')).endswith('-fixed'):
            raise ValueError('Expected the editable v102 runtime, capability 88.')
        run([sys.executable,str(launcher/'build_launcher_exe.py'),'--folder',str(launcher)],cwd=launcher,check=True)
        built=launcher/'launcher-bin/ReRAC_Weapon_Evolution.exe';verify(built)
        shutil.copytree(playable/'runtime',staged);shutil.copy2(game,staged/'rerac.exe')
        shutil.copytree(source/'crates/rc-engine/assets/shaders',staged/'assets/shaders',dirs_exist_ok=True)
        (staged/'build-info.json').write_text(json.dumps({'patch':PATCH,'version':version},indent=2))
        (playable/'runtime').rename(previous)
        try:staged.rename(playable/'runtime')
        except Exception:previous.rename(playable/'runtime');raise
        switched=True;write(target,built.read_bytes())
        if hashlib.sha256(target.read_bytes()).digest()!=hashlib.sha256(built.read_bytes()).digest():raise ValueError('Installed launcher does not match the verified build.')
        verify(target)
        if (built.parent/'licenses').is_dir():shutil.copytree(built.parent/'licenses',playable/'licenses',dirs_exist_ok=True)
        write(bundle/'installed.json',json.dumps({'playable':str(playable),'launcher':str(launcher),'exe':str(target)},indent=2).encode())
    except Exception:
        if switched:
            failed=playable/('.failed_v102_'+str(time.time_ns()));(playable/'runtime').rename(failed);previous.rename(playable/'runtime');shutil.rmtree(failed)
        for path,data in originals.items():
            if data is None:path.unlink(missing_ok=True)
            else:write(path,data)
        if staged.exists():shutil.rmtree(staged)
        print('Update failed. Original sources, profiles and playable EXEs restored.',flush=True)
        raise
    return backup

def choose_folder(prompt,default=''):
    return Path(input(prompt).strip().strip('"') or default).resolve()
def main():
    if os.name!='nt':raise ValueError('Run this local update on Windows.')
    print('Weapon Evolution v102 local test update. Close the game and launcher. No release is published.')
    source=choose_folder(r'EXISTING ReRAC Rust SOURCE folder [C:\ReRAC-Mod\rerac]: ',r'C:\ReRAC-Mod\rerac')
    playable=choose_folder('EXISTING PLAYABLE mod folder (EXE, weapons.ini and runtime): ')
    if not (source/'Cargo.toml').is_file() or not (playable/'weapons.ini').is_file() or not (playable/'runtime/rerac.exe').is_file():raise ValueError('Select the existing Rust source and playable folders, not this update kit.')
    launcher=next((p for p in [playable,playable/'launcher',playable/'source/launcher'] if (p/'launcher.py').is_file() and (p/'build_launcher_exe.py').is_file()),None)
    if launcher is None:
        selected=choose_folder('CURRENT launcher SOURCE folder (launcher.py and build_launcher_exe.py): ')
        launcher=next((p for p in [selected,selected/'launcher',selected/'source/launcher'] if (p/'build_launcher_exe.py').is_file()),None)
    if launcher is None:raise ValueError('Launcher source folder not found.')
    target=playable/'ReRAC_Weapon_Evolution.exe'
    if not target.is_file():target=choose_folder('Full path of the launcher EXE you actually open: ')
    if target.parent!=playable or target.suffix.casefold()!='.exe' or not target.is_file():raise ValueError('Select the EXE inside the playable mod folder.')
    plan(source,launcher)
    for name in ['rerac.exe',target.name]:
        processes=subprocess.run(['tasklist','/FI','IMAGENAME eq '+name,'/FO','CSV','/NH'],capture_output=True,text=True,errors='replace',check=True)
        if ('"'+name.casefold()+'"') in processes.stdout.casefold():raise ValueError('Close '+name+' before updating.')
    ensure_build_environment();backup=install(source,playable,launcher,target)
    print('INSTALLED runtime v102 and launcher 0.28.7: '+str(target));print('Backup: '+str(backup))
    print('Use 02_CREATE_PLAYABLE_ZIP.bat to package this tested installation.');os.startfile(target)
if __name__=='__main__':
    try:main()
    except Exception as error:print('ERROR: '+str(error));sys.exit(1)
