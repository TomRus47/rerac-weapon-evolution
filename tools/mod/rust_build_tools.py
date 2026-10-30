"""Verify Rust before Cargo cache changes; recover in a private toolchain root.

The user's installed toolchain is never removed, edited or repaired in place.
Only this installer process and its children receive the selected environment.
No security settings or persistent user environment variables are changed.
"""
from pathlib import Path
import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from rust_download import install_toolchain

CHANNEL='stable-x86_64-pc-windows-msvc'


def put(env,key,value):
    for old in list(env):
        if old.casefold()==key.casefold():del env[old]
    env[key]=str(value)


def value(env,key):
    return next((v for k,v in env.items() if k.casefold()==key.casefold()),'')


def run(args,env,source):
    result=subprocess.run([str(a) for a in args],cwd=source,env=env,
        capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=120)
    if result.returncode:
        raise ValueError((result.stderr or result.stdout or 'Process failed.').strip())
    return result.stdout.strip()


def probe(compiler,env,source):
    """Execute the real binary and compile metadata, not just a Rustup proxy."""
    version=run([compiler,'--version'],env,source)
    if not version.startswith('rustc '):raise ValueError('Unexpected Rust compiler identity: '+version)
    with tempfile.TemporaryDirectory(prefix='rerac-rust-probe-') as folder:
        path=Path(folder);code=path/'compiler_probe.rs'
        code.write_text('pub fn compiler_probe() -> usize { std::mem::size_of::<usize>() }\n')
        run([compiler,'--crate-name','rerac_compiler_probe','--crate-type','lib',
             '--emit=metadata',code,'-o',path/'compiler_probe.rmeta'],env,source)
        if not (path/'compiler_probe.rmeta').is_file():raise ValueError('Rust did not produce probe metadata.')
    return version


def checked_environment(rustup,env,source):
    env=dict(env)
    compiler=Path(run([rustup,'which','--toolchain',CHANNEL,'rustc'],env,source))
    cargo=Path(run([rustup,'which','--toolchain',CHANNEL,'cargo'],env,source))
    if not compiler.is_file() or not cargo.is_file():raise ValueError('Rustup reports missing compiler/Cargo files.')
    put(env,'RUSTUP_TOOLCHAIN',CHANNEL)
    put(env,'RUSTC',compiler)
    put(env,'PATH',str(compiler.parent)+os.pathsep+value(env,'PATH'))
    version=probe(compiler,env,source)
    if not run([cargo,'--version'],env,source).startswith('cargo '):raise ValueError('Unexpected Cargo identity.')
    return env,version


def ensure_rust_compiler(source,bundle):
    """Prefer working installed Rust; download a separate copy only if needed."""
    source,bundle=Path(source).resolve(),Path(bundle).resolve()
    initial=dict(os.environ)
    rustup=shutil.which('rustup',path=value(initial,'PATH'))
    if not rustup:raise ValueError('Rustup was not found. Install Rustup before running this source kit.')
    # Resolve the proxy now, before a selected toolchain prepends its own bin.
    rustup=str(Path(rustup).resolve())
    try:
        env,version=checked_environment(rustup,initial,source)
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        print('Installed Rust could not execute the compiler probe: '+str(error),flush=True)
    else:
        os.environ.clear();os.environ.update(env)
        print('Rust compiler verified: '+version,flush=True)
        return
    tools=bundle/'build-tools'
    # Reuse only explicitly marked compiler caches from sibling Weapon Evolution
    # test kits. This avoids downloading Rust again for every gameplay hotfix.
    cache_tools=[tools]
    siblings=sorted((kit for kit in bundle.parent.glob('ReRAC_Weapon_Evolution_v*')
        if kit.is_dir() and not kit.is_symlink() and kit.resolve()!=bundle
        and (kit/'build-tools/verified-rust.json').is_file()),
        key=lambda kit:kit.name,reverse=True)
    cache_tools.extend(kit/'build-tools' for kit in siblings[:32])
    for cached in cache_tools:
        try:
            if cached.is_symlink():continue
            record=json.loads((cached/'verified-rust.json').read_text(encoding='utf-8'));name=record['root']
            if not isinstance(name,str) or not re.fullmatch(r'rustup-\d+',name):continue
            home=cached/name
            if home.is_symlink() or home.resolve().parent!=cached.resolve():continue
            env=dict(initial);put(env,'RUSTUP_HOME',home);put(env,'RUSTUP_TOOLCHAIN',CHANNEL)
            env,version=checked_environment(rustup,env,source)
        except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError):
            continue
        os.environ.clear();os.environ.update(env)
        print('Reusing verified Rust toolchain: '+str(home),flush=True)
        print('Separate Rust compiler verified: '+version,flush=True)
        return
    tools.mkdir(exist_ok=True)
    marker=tools/'verified-rust.json'
    home=tools/('rustup-'+str(time.time_ns()))
    env=dict(initial);put(env,'RUSTUP_HOME',home);put(env,'RUSTUP_TOOLCHAIN',CHANNEL)
    print('Installing a separate Rust toolchain for this kit: '+str(home),flush=True)
    print('Internet is required. The existing Rust installation and Cargo dependency cache are retained.',flush=True)
    install_toolchain(rustup,CHANNEL,env,source)
    try:
        env,version=checked_environment(rustup,env,source)
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        raise ValueError('The separate Rust compiler also failed its execution/compile probe. '
            'The playable installation and Cargo build cache have not been changed. '
            'Read this log for the exact Windows/Rustup error: '+str(error)) from error
    marker.write_text(json.dumps({'root':home.name,'channel':CHANNEL},indent=2)+'\n')
    os.environ.clear();os.environ.update(env)
    print('Separate Rust compiler verified: '+version,flush=True)
