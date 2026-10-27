"""Initialize installed x64 compiler tools for this process and its children."""
from pathlib import Path
import os,re,shutil,subprocess

def value(env,key):return next((v for k,v in env.items() if k.casefold()==key.casefold()),'')

def parse(output,base):
    env=dict(base)
    for line in output.splitlines():
        key,sep,val=line.partition('=')
        if sep and re.fullmatch(r'[A-Za-z_][A-Za-z0-9_() -]*',key):
            for old in list(env):
                if old.casefold()==key.casefold():del env[old]
            env[key]=val
    return env

def commands(path,devcmd):
    path=str(path)
    if any(c in path for c in '\r\n"%'):raise ValueError('Unsupported compiler setup path')
    return '@echo off\ncall "'+path+'"'+(' -arch=x64 -host_arch=x64' if devcmd else '')+' >nul 2>&1\nif errorlevel 1 exit 1\nwhere cl >nul 2>&1\nif errorlevel 1 exit 2\nset\nexit 0\n'

def ensure_build_environment():
    if os.name!='nt':raise ValueError('Run this updater on Windows.')
    env=dict(os.environ);path=value(env,'PATH');cargo=Path(value(env,'USERPROFILE'))/'.cargo/bin'
    if not shutil.which('cargo',path=path) and (cargo/'cargo.exe').is_file():env['PATH']=str(cargo)+os.pathsep+path
    arch=value(env,'VSCMD_ARG_TGT_ARCH') or value(env,'Platform')
    ready=shutil.which('cl',path=value(env,'PATH')) and arch.lower()=='x64'
    roots=[]
    if not ready:
        for key in ['ProgramFiles(x86)','ProgramFiles']:
            if not value(env,key):continue
            vswhere=Path(value(env,key))/'Microsoft Visual Studio/Installer/vswhere.exe'
            if vswhere.is_file():
                r=subprocess.run([str(vswhere),'-latest','-products','*','-requires','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-property','installationPath'],capture_output=True,text=True,errors='replace',check=False)
                if r.returncode==0:roots.extend(Path(p.strip()) for p in r.stdout.splitlines() if p.strip())
        if value(env,'VSINSTALLDIR'):roots.append(Path(value(env,'VSINSTALLDIR')))
        for key in ['ProgramFiles','ProgramFiles(x86)']:
            if value(env,key):
                roots.extend(Path(value(env,key))/'Microsoft Visual Studio'/year/edition for year in ['2022','2019'] for edition in ['Community','Professional','Enterprise','BuildTools'])
        for root in roots:
            for rel,devcmd in [('Common7/Tools/VsDevCmd.bat',True),('VC/Auxiliary/Build/vcvars64.bat',False)]:
                setup=root/rel
                if not setup.is_file():continue
                print('Initializing Visual Studio x64 tools: '+str(setup),flush=True)
                r=subprocess.run([value(env,'COMSPEC') or 'cmd.exe','/d','/q'],input=commands(setup,devcmd),capture_output=True,text=True,errors='replace',env=env,check=False)
                if r.returncode:continue
                candidate=parse(r.stdout,env)
                if shutil.which('cl',path=value(candidate,'PATH')):env=candidate;ready=True;break
            if ready:break
    if not ready:raise ValueError('Visual Studio x64 tools could not be initialized. Enable Desktop development with C++ and a Windows SDK in Visual Studio Installer.')
    if not shutil.which('cargo',path=value(env,'PATH')):raise ValueError('Rust/Cargo was not found on PATH or in the user .cargo/bin directory.')
    os.environ.clear();os.environ.update(env)
    print('Windows x64 build environment ready.',flush=True)
