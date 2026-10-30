"""Use Cargo's reported executable and retain a previously verified GUI build.

No target-layout guesses, broad executable searches or relaxed version checks.
Prepared-source identity comes from the bundled development descriptor, so
new patch exports cannot accidentally retain a previous version check.
"""
from __future__ import annotations
import hashlib
import json
import re
from pathlib import Path
import subprocess


def sha(data):
    return hashlib.sha256(data).hexdigest()


def artifact_from_message(message, source):
    """Accept only the bin produced for this exact rc-engine manifest."""
    if message.get('reason') != 'compiler-artifact':
        return None
    target = message.get('target', {})
    if target.get('name') != 'rerac' or 'bin' not in target.get('kind', []):
        return None
    manifest = message.get('manifest_path')
    if not manifest or Path(manifest).resolve() != (source / 'crates/rc-engine/Cargo.toml').resolve():
        return None
    executable = message.get('executable')
    return Path(executable).resolve() if executable else None


def cargo_build(source, target):
    """Stream diagnostics, record the emitted executable, and honor Cargo errors."""
    args = ['cargo', 'build', '--release', '--locked',
            '--no-default-features', '--manifest-path', str(source / 'Cargo.toml'),
            '--target-dir', str(target), '-p', 'rc-engine', '--bin', 'rerac',
            '--message-format=json-render-diagnostics']
    print(subprocess.list2cmdline(args), flush=True)
    process = subprocess.Popen(args, cwd=source, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, encoding='utf-8', errors='replace', bufsize=1)
    artifacts = set()
    try:
        for line in process.stdout:
            try:
                message = json.loads(line)
            except (ValueError, TypeError):
                print(line, end='', flush=True)
                continue
            if not isinstance(message, dict):
                print(line, end='', flush=True)
                continue
            artifact = artifact_from_message(message, source)
            if artifact is not None:
                artifacts.add(artifact)
            if message.get('reason') == 'compiler-message':
                rendered = message.get('message', {}).get('rendered')
                if rendered:
                    print(rendered, end='' if rendered.endswith('\n') else '\n', flush=True)
        code = process.wait()
        if code:
            raise subprocess.CalledProcessError(code, args)
    except BaseException:
        if process.poll() is None:
            process.terminate()
            process.wait()
        raise
    finally:
        process.stdout.close()
    if len(artifacts) != 1:
        raise ValueError('Cargo did not report exactly one executable for this rc-engine source: '
                         + repr(sorted(map(str, artifacts))))
    executable = artifacts.pop()
    if not executable.is_file():
        raise ValueError('Cargo-reported executable is missing: ' + str(executable))
    print('Cargo-reported runtime: ' + str(executable), flush=True)
    return executable


def runtime_identity(executable):
    result = subprocess.run([str(executable), '--version-json'], check=True,
                            capture_output=True, text=True, encoding='utf-8', timeout=30)
    try:
        identity = json.loads(result.stdout.strip())
    except ValueError as error:
        raise ValueError('Runtime did not return valid version JSON: ' + result.stdout) from error
    if not isinstance(identity, dict):
        raise ValueError('Runtime version JSON is not an object: ' + repr(identity))
    print('Actual runtime identity: ' + json.dumps(identity, sort_keys=True), flush=True)
    return identity


def run_text(args, source):
    """Keep compiler/test output in the visible console and the resume log."""
    print(subprocess.list2cmdline(args), flush=True)
    process = subprocess.Popen(args, cwd=source, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, encoding='utf-8', errors='replace', bufsize=1)
    try:
        for line in process.stdout:
            print(line, end='', flush=True)
        code = process.wait()
        if code:
            raise subprocess.CalledProcessError(code, args)
    except BaseException:
        if process.poll() is None:
            process.terminate()
            process.wait()
        raise
    finally:
        process.stdout.close()


def workspace_packages(source):
    """Validate the local dependency paths before selecting cache entries."""
    result = subprocess.run(['cargo', 'metadata', '--format-version', '1',
        '--no-deps', '--locked', '--manifest-path', str(source / 'Cargo.toml')],
        cwd=source, check=True, capture_output=True, text=True, encoding='utf-8', errors='replace')
    record = json.loads(result.stdout)
    packages = {p['id']: p for p in record.get('packages', [])}
    members = record.get('workspace_members', [])
    if not members or any(p not in packages for p in members):
        raise ValueError('Cargo did not report all local workspace packages.')
    selected = []
    for identifier in members:
        package = packages[identifier]
        manifest = Path(package['manifest_path']).resolve()
        if not manifest.is_relative_to(source) or not manifest.is_file():
            raise ValueError('Workspace package lies outside the prepared source: ' + str(manifest))
        selected.append(package)
    by_name = {p['name']: p for p in selected}
    if len(by_name) != len(selected) or not {'rc-game', 'rc-engine', 'rc-formats'}.issubset(by_name):
        raise ValueError('Incomplete or ambiguous ReRAC workspace metadata.')
    for name in ('rc-game', 'rc-engine', 'rc-formats'):
        expected = source / 'crates' / name / 'Cargo.toml'
        if Path(by_name[name]['manifest_path']).resolve() != expected.resolve():
            raise ValueError('Unexpected prepared manifest for ' + name)
    game = next((d for d in by_name['rc-engine'].get('dependencies', []) if d['name'] == 'rc-game'), None)
    if not game or not game.get('path') or Path(game['path']).resolve() != (source / 'crates/rc-game').resolve():
        raise ValueError('rc-engine does not depend on this prepared rc-game source.')
    return sorted(by_name)


def validate_source_apis(source):
    """A clean cache cannot repair missing source; reject that separately."""
    source = Path(source).resolve()
    identity = json.loads((source / 'mod/development.json').read_text(encoding='utf-8'))
    patch = identity.get('patch')
    capability = identity.get('config_capability')
    if (not isinstance(patch, str) or not re.fullmatch(r'0\.1\.6\.\d+', patch)
            or type(capability) is not int or capability < 92):
        raise ValueError('Invalid prepared-source development identity.')
    required = {
        'crates/rc-engine/src/disc_source.rs':
            ('"weapon_evolution_patch":"' + patch + '"',
             '"weapon_evolution_config":' + str(capability)),
        'crates/rc-game/src/weapon_evo.rs': ('pubmodhealth_test;',),
        'crates/rc-game/src/hud.rs': ('pubmodnanotech;', 'Nanotech16{'),
        'crates/rc-game/src/weapon_evo_notice.rs': ('[u16;37]',),
        'crates/rc-game/src/weapon_evo_upgrade_wave.rs': ('[u16;37]',),
    }
    for name, markers in required.items():
        text = re.sub(r'\s+', '', (source / name).read_text(encoding='utf-8'))
        for marker in markers:
            if marker not in text:
                raise ValueError('Prepared ' + patch + ' source is incomplete: ' + name + ' (missing ' + marker + ')')


def build_engine(source, target, validate):
    """Rebuild the whole local workspace interface, then test and link it.

    ZIP extraction can preserve timestamps older than a shared Cargo fingerprint.
    Forcing only rc-engine leaves stale rc-game rlibs/rmeta usable. Clean every
    local workspace package in the release cache, preserving third-party packages.
    Previous cached Rust test results are deliberately not reused after this bug.
    """
    source, target = Path(source).resolve(), Path(target).resolve()
    validate_source_apis(source)
    packages = workspace_packages(source)
    clean = ['cargo', 'clean', '--release', '--manifest-path', str(source / 'Cargo.toml'),
             '--target-dir', str(target)]
    for name in packages:
        clean += ['-p', name]
    print('Discarding compiled LOCAL workspace packages: ' + ', '.join(packages), flush=True)
    print('Third-party dependency cache, source files and installed runtime are retained.', flush=True)
    run_text(clean, source)
    common = ['--release', '--locked', '--no-default-features',
              '--manifest-path', str(source / 'Cargo.toml'), '--target-dir', str(target)]
    for name in ('rc-formats', 'rc-game'):
        run_text(['cargo', 'test', *common, '-p', name, '--lib'], source)
    executable = cargo_build(source, target)
    identity = runtime_identity(executable)
    try:
        validate(identity)
    except ValueError as error:
        raise ValueError('Fresh workspace runtime fails the declared identity check. '
                         'No playable files were replaced. Actual identity: ' + json.dumps(identity)) from error
    if identity.get('data_format') != 1:
        raise ValueError('Unexpected runtime data format; no playable files were replaced.')
    return executable


def checked_launcher(stage, builder):
    """Builder metadata is written only after its standalone self-test passes.

    Bind that result to both current inputs and current EXE bytes; never reuse
    a filename alone as proof. Cache misses retain the normal full build/test.
    """
    stage = Path(stage)
    executable = stage / 'launcher-bin' / builder.EXE_NAME
    metadata = executable.parent / 'launcher-build.json'
    if not executable.is_file() or not metadata.is_file():
        return None
    try:
        saved = json.loads(metadata.read_text(encoding='utf-8'))
        if (saved.get('launcher_version') != builder.core.VERSION
                or saved.get('sources_sha256') != builder.source_digest(stage)
                or saved.get('exe_sha256') != sha(executable.read_bytes())):
            return None
        builder.windows_x64(executable)
    except (OSError, ValueError, TypeError):
        return None
    print('Reusing the matching standalone launcher; its successful self-test is retained.', flush=True)
    return executable
