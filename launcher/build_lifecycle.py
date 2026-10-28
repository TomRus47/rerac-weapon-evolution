"""Bounded Windows file-lock retries and durable standalone-build diagnostics.

An EXE scanner or a child process can temporarily hold a freshly linked file.
Retry only file-access errors, never a failed application self-test. Keep failed
builds for inspection; cleanup must not hide their original exception.
"""
from contextlib import contextmanager
from pathlib import Path
import json
import shutil
import subprocess
import tempfile
import time
import traceback


def retry_file_access(operation, description, delays=(1, 2, 4, 8)):
    for attempt in range(len(delays) + 1):
        try:
            return operation()
        except OSError as error:
            transient = isinstance(error, PermissionError) or getattr(error, 'winerror', None) in (5, 32, 33)
            if not transient or attempt == len(delays):
                raise
            delay = delays[attempt]
            print(f'{description}: file access is temporarily denied. Retrying in {delay}s.', flush=True)
            time.sleep(delay)


@contextmanager
def retained_build_stage(build_root):
    build_root.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix='build_', dir=build_root))
    try:
        yield stage
    except BaseException:
        print('Build files retained for diagnosis: ' + str(stage), flush=True)
        raise
    else:
        # A verified output has already been copied to launcher-bin. A Windows
        # lock on a disposable file must not turn that success into a failure.
        try:
            shutil.rmtree(stage)
        except OSError as error:
            print(f'Build succeeded; temporary files could not yet be removed: {stage}\n{error}', flush=True)


def run_self_test(exe, report, *, timeout=600, poll_seconds=10):
    """Run the real frozen GUI check, preserving reports even on launch failure."""
    exe, report = Path(exe).resolve(), Path(report).resolve()
    report.parent.mkdir(parents=True, exist_ok=True)
    progress = report.with_suffix('.progress.json')
    process_log = report.with_suffix('.process.log')
    command = [str(exe), '--self-test', str(report)]
    # Callers use a fresh report name. Refuse stale proof if invoked directly.
    for path in (report, progress, report.with_suffix('.error.log')):
        path.unlink(missing_ok=True)
    process = None
    print('Verifying all preloaded launcher pages. Please leave the test window alone.', flush=True)
    print('Persistent self-test report: ' + str(report), flush=True)
    try:
        with process_log.open('w', encoding='utf-8') as stream:
            process = retry_file_access(
                lambda: subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT),
                'Starting the standalone launcher self-test')
            started = time.monotonic()
            while True:
                remaining = timeout - (time.monotonic() - started)
                if remaining <= 0:
                    raise subprocess.TimeoutExpired(command, timeout)
                try:
                    code = process.wait(timeout=min(poll_seconds, remaining))
                    break
                except subprocess.TimeoutExpired:
                    stage = 'initializing the frozen Python/Tk application'
                    try:
                        stage = json.loads(progress.read_text(encoding='utf-8')).get('stage', stage)
                    except (OSError, ValueError):
                        pass
                    print(f'Self-test is running ({int(time.monotonic()-started)}s): {stage}', flush=True)
            if code:
                detail = ''
                try:
                    detail = json.loads(report.read_text(encoding='utf-8')).get('error', '')
                except (OSError, ValueError):
                    pass
                raise ValueError(f'Launcher self-test failed (exit {code}). {detail}\nReport: {report}')
        value = json.loads(report.read_text(encoding='utf-8'))
        if value.get('ok') is False:
            raise ValueError(f'Launcher self-test reported an error: {value.get("error", "unknown error")}\nReport: {report}')
        return value
    except BaseException as error:
        if process is not None and process.poll() is None:
            # Stop this self-test only. Never terminate the player's launcher
            # or game by executable name; any remaining child files are retained.
            try:
                process.kill()
                process.wait(timeout=5)
            except (OSError, subprocess.TimeoutExpired) as stop_error:
                print('Could not finish stopping the self-test; build files will be retained: '+str(stop_error), flush=True)
        if not report.exists():
            report.write_text(json.dumps({'ok': False, 'phase': 'standalone self-test',
                'error': str(error), 'traceback': traceback.format_exc(),
                'executable': str(exe), 'process_log': str(process_log)}, indent=2)+'\n', encoding='utf-8')
        raise
