"""Bounded Rustup transport fallback for reset/failed HTTPS downloads.

Use Rustup's supported alternate cURL backend only after a network failure.
Keep the original distribution server, proxy configuration, TLS verification
and Rustup checksums. Environment changes belong to child processes only.
"""
from collections import deque
import os
import subprocess


def _set(env,key,value=None):
    for old in list(env):
        if old.casefold()==key.casefold():del env[old]
    if value is not None:env[key]=value


def _value(env,key):
    return next((v for k,v in env.items() if k.casefold()==key.casefold()),'')


def _network_failure(text):
    text=text.lower()
    return any(marker in text for marker in (
        'could not download','error downloading','error sending request',
        'connection reset','os error 10054','could not resolve host',
        'could not connect','failed to connect','timed out','network is unreachable'))


def _attempt(args,env,source):
    # Stream progress into the same diagnostic log as the normal installer.
    # Retain only a bounded tail for deciding whether a transport retry helps.
    process=subprocess.Popen(args,cwd=source,env=env,stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,text=True,encoding='utf-8',errors='replace',bufsize=1)
    tail=deque(maxlen=100)
    try:
        for line in process.stdout:
            print(line,end='',flush=True);tail.append(line[-2048:])
        code=process.wait()
        return code,''.join(tail)
    except BaseException:
        if process.poll() is None:
            process.terminate();process.wait()
        raise
    finally:
        process.stdout.close()


def install_toolchain(rustup,channel,env,source):
    """Try configured HTTPS transport, then the other Rustup backend once."""
    args=[rustup,'toolchain','install',channel,'--profile','minimal']
    primary=dict(env)
    if not _value(primary,'RUSTUP_MAX_RETRIES'):_set(primary,'RUSTUP_MAX_RETRIES','2')
    code,detail=_attempt(args,primary,source)
    if code==0:return
    if not _network_failure(detail):
        raise ValueError('Rustup installation failed before compilation: '+detail.strip())
    alternate=dict(primary)
    if _value(primary,'RUSTUP_USE_CURL')=='1':
        _set(alternate,'RUSTUP_USE_CURL')
        label='default HTTPS backend'
    else:
        _set(alternate,'RUSTUP_USE_CURL','1')
        label='alternate cURL HTTPS backend'
    print('Rustup network download failed. Retrying once with '+label+'.',flush=True)
    code,detail=_attempt(args,alternate,source)
    if code==0:return
    raise ValueError('Rust could not be downloaded with either Rustup transport. '
        'The game and Cargo build cache have not been changed. '
        'Check access to https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256 '
        'in a browser, or retry on another connection. '
        'No security or certificate settings were changed. Last Rustup output: '+detail.strip())
