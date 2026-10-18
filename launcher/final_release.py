"""Install the approved default once, then build the final standalone Windows release."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import launcher_core as core

def install_default(kit):
    source=kit/'weapons.default.ini';raw=source.read_bytes()
    core.profile_document(raw)
    fingerprint=hashlib.sha256(raw).hexdigest()
    marker=kit/'defaults-installed-v05.json'
    if marker.is_file():
        value=json.loads(marker.read_text(encoding='utf-8'))
        if value.get('sha256')==fingerprint:return False
    target=kit/'weapons.ini'
    if target.is_file() and target.read_bytes()!=raw:
        backup=kit/'backup/defaults_before_v05'/core.stamp()/'weapons.ini'
        core.atomic_write(backup,target.read_bytes())
        print('Previous default backed up: '+str(backup),flush=True)
    core.atomic_write(target,raw)
    core.atomic_write(marker,(json.dumps({'release':'0.5','sha256':fingerprint},indent=2)+'\n').encode('utf-8'))
    print('Approved weapons.ini default installed. Future manual edits will be preserved.',flush=True)
    return True

def main():
    try:
        install_default(core.KIT)
        import package_launcher
        return package_launcher.main()
    except (OSError,ValueError,UnicodeError,subprocess.SubprocessError) as error:
        print('ERROR: '+str(error),file=sys.stderr);return 2

if __name__=='__main__':raise SystemExit(main())
