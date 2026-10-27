"""Apply verified v105 source blocks, test/build runtime v106, retain the launcher."""
from pathlib import Path,PurePosixPath
import hashlib,json,os,shutil,subprocess,sys,time
from windows_build_env import ensure_build_environment
HERE=Path(__file__).resolve().parent
PATCH='0.1.6.106'
def digest(b):return hashlib.sha256(b.replace(b'\r\n',b'\n')).hexdigest()
def write(path,data):
    path.parent.mkdir(parents=True,exist_ok=True);temp=path.with_name(path.name+'.v106.tmp')
    try:temp.write_bytes(data);os.replace(temp,path)
    finally:temp.unlink(missing_ok=True)
def safe(source,name):
    rel=PurePosixPath(name)
    if rel.is_absolute() or '..' in rel.parts or '\\' in name or ':' in name:raise ValueError('Unsafe patch path.')
    target=source.joinpath(*rel.parts)
    if target.is_symlink() or not target.resolve().is_relative_to(source.resolve()):raise ValueError('Source path escapes the selected folder.')
    return target

def plan(source,bundle=HERE):
    data=json.loads((bundle/'manifest.json').read_text());changes={};texts={};raws={}
    if data.get('patch')!=PATCH:raise ValueError('Extract the complete v106 kit.')
    for row in data['edits']:
        path=safe(source,row['path'])
        if path not in texts:
            raws[path]=path.read_bytes();texts[path]=raws[path].decode('utf-8').replace('\r\n','\n')
        text=texts[path]
        if 'append' in row:
            if row['append'] in text:continue
            if text.count(row['marker'])!=1:raise ValueError('Unknown source interface: '+row['path'])
            text+=row['append']
        elif text.count(row['after'])==row['count']:continue
        elif text.count(row['before'])==row['count']:text=text.replace(row['before'],row['after'])
        else:raise ValueError('Unrecognized v105 source block: '+row['path']+'. No files changed.')
        texts[path]=text
    for path,text in texts.items():
        data=text.replace('\n','\r\n').encode() if b'\r\n' in raws[path] else text.encode()
        if data!=raws[path]:changes[path]=data
    for row in data_manifest(bundle)['files']:
        target=safe(source,row['path']);body=safe(bundle/'payload',row['file']).read_bytes()
        if digest(body)!=row['sha256']:raise ValueError('Damaged payload.')
        if target.exists():
            current=digest(target.read_bytes())
            if current==row['sha256']:continue
            if current!=row.get('before_sha256'):raise ValueError('Unknown existing test file: '+row['path'])
        changes[target]=body
    return changes
def data_manifest(bundle):return json.loads((bundle/'manifest.json').read_text())
def install(source,playable,launcher,bundle=HERE,run=subprocess.run):
    changes=plan(source,bundle);backup=playable/('backup_v106_'+str(time.time_ns()));backup.mkdir()
    originals={p:p.read_bytes() if p.is_file() else None for p in changes}
    runtime=playable/'runtime/rerac.exe';old_runtime=backup/'rerac.exe';shutil.copy2(runtime,old_runtime)
    for i,(path,body) in enumerate(originals.items()):
        if body is not None:(backup/(str(i)+'-'+path.name)).write_bytes(body)
    (backup/'restore.json').write_text(json.dumps([{'path':str(p),'backup':str(i)+'-'+p.name if body is not None else None} for i,(p,body) in enumerate(originals.items())],indent=2))
    try:
        for path,body in changes.items():write(path,body)
        common=['--release','--locked','--no-default-features','--target-dir',str(source/'target')]
        for args in [['cargo','test',*common,'-p','rc-game','--lib'],['cargo','build',*common,'-p','rc-engine','--bin','rerac']]:
            print(subprocess.list2cmdline(args),flush=True);run(args,cwd=source,check=True)
        exe=source/'target/release/rerac.exe'
        version=json.loads(run([str(exe),'--version-json'],capture_output=True,text=True,check=True,timeout=30).stdout)
        if version.get('weapon_evolution_patch')!=PATCH or version.get('weapon_evolution_config',0)<89 or str(version.get('version','')).endswith('-fixed'):
            raise ValueError('Expected editable v106 runtime, configuration capability 89.')
        staged=runtime.with_name('rerac.v106-stage.exe');shutil.copy2(exe,staged);os.replace(staged,runtime)
        (bundle/'installed.json').write_text(json.dumps({'playable':str(playable),'launcher_exe':str(launcher),'patch':PATCH},indent=2))
    except Exception:
        for path,body in originals.items():
            if body is None:path.unlink(missing_ok=True)
            else:write(path,body)
        shutil.copy2(old_runtime,runtime)
        print('Update failed. Source and playable runtime restored. Configuration, saves and launcher retained.',flush=True)
        raise
    return backup

def folder(prompt,default=''):return Path(input(prompt).strip().strip('"') or default).resolve()
def main():
    if os.name!='nt':raise ValueError('Run this author build kit on Windows.')
    print('Weapon Evolution v106 local test. Close the game and launcher. No GitHub operations.')
    source=folder(r'EXISTING v105 ReRAC SOURCE folder [C:\ReRAC-Mod\rerac]: ',r'C:\ReRAC-Mod\rerac')
    playable=folder('EXISTING playable mod folder (launcher EXE, weapons.ini and runtime): ')
    launcher=playable/'ReRAC_Weapon_Evolution.exe'
    if not launcher.is_file():launcher=folder('Full path of your existing mod launcher EXE: ')
    if not (source/'Cargo.toml').is_file() or not (playable/'weapons.ini').is_file() or not (playable/'runtime/rerac.exe').is_file() or launcher.parent!=playable or not launcher.is_file():
        raise ValueError('Select the existing source and playable mod folders, not this kit.')
    plan(source)
    for name in ['rerac.exe',launcher.name]:
        result=subprocess.run(['tasklist','/FI','IMAGENAME eq '+name,'/FO','CSV','/NH'],capture_output=True,text=True,errors='replace',check=True)
        if ('"'+name.casefold()+'"') in result.stdout.casefold():raise ValueError('Close '+name+' before updating.')
    ensure_build_environment();backup=install(source,playable,launcher)
    print('Installed runtime '+PATCH+'. Launcher 0.28.9 retained. Backup: '+str(backup));os.startfile(launcher)
if __name__=='__main__':
    try:main()
    except Exception as error:print('ERROR: '+str(error));sys.exit(1)
