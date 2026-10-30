"""Export the complete v118 local source test, excluding builds and user data."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile
from create_nanotech116_kit import batch
from verified_artifacts import validate_source_apis

ROOT=Path(__file__).resolve().parents[2]
NAME='ReRAC_Weapon_Evolution_v118_Orxon_Shortcut_Test'
README='''Weapon Evolution v118 / ReRAC 0.3.8-alpha — ORXON SHORTCUT FIX 118

This includes all v117 gameplay fixes and the working 117.3 installer recovery.
The Orxon shortcut now recognizes the actual relocated lift update (class 1141),
initializes before scheduler culling, and waits below until Clank boards.
This applies ONLY to Randomizer All, with the actual O2 Mask already owned and
Clank-side checks still pending. Other modes, gates and mission flags are unchanged.
No new save is needed. Reload your save or leave/re-enter Orxon after updating.
Keep the extracted v117.3 folder nearby: its marked working Rust compiler cache
can be verified/reused automatically, without downloading Rust again.
Before cleaning any Cargo artifacts, the installer executes the real Rust compiler
and compiles a small metadata probe. If installed Rust is missing, broken or
cannot start, it automatically downloads a separate minimal MSVC toolchain under
this kit's build-tools folder using the installed Rustup. Internet is required.
The original Rust installation is not removed or edited. A verified local copy
is reused on a later attempt, including marked caches in sibling mod test kits. Windows must permit the new compiler to execute;
if that fails too, the installer stops with its diagnostic before cache cleaning.
If the Rustup HTTPS download fails (for example, Windows connection reset 10054),
the installer retries once with Rustup's alternative supported HTTPS backend.
TLS certificate checks, official distribution URLs and Rustup checksum checks
are retained. A network/domain block can still prevent both download methods;
in that case the diagnostic names the endpoint to test in a browser.
Gameplay, launcher version, profile values and saves are unchanged.

1. Extract this entire ZIP into a new writable folder. Close the game and mod
   launcher. Run 01_COMPILE_AND_UPDATE.bat.
2. Select your EXISTING playable mod folder containing weapons.ini,
   ReRAC_Weapon_Evolution.exe and runtime/rerac.exe. Do not select this new kit
   or its source folder. The updater compiles the bundled sources; applying
   previous individual source patches is unnecessary.
3. Wait for Rust tests, runtime compilation and the launcher GUI self-test.
   The self-test checks every language and preloaded page and can take minutes.
   Do not close the window. On success, the updated launcher opens automatically.

Requirements: Python 3.10+, Rust/Cargo, Visual Studio C++ x64 build tools and a
Windows SDK. Installed x64 tools are initialized automatically. Uncached Cargo
packages need internet. A cold runtime build may take a long time.

The runtime identity must be ReRAC 0.3.8 / patch 0.1.6.118 / capability 93.
The launcher is 0.29.6. Tests, identity and GUI checks complete before replacing
installed files. A failure retains the previous installation. Runtime/shaders,
launcher and any migrated profile changes have rollback backups. The custom
launcher icon, existing INI balance, presets, saves and randomizer seeds survive.
One new setting is added by the editor: enemy_burst_tracking, initially Off.
Find it under Global settings > Encounter difficulty and Nanotech.

02_OPEN_UPDATED_LAUNCHER.bat reopens the SAME installation after a successful
update. 03_CREATE_PLAYABLE_ZIP.bat packages that runtime, standalone launcher,
shaders and current weapons.ini, excluding personal presets/saves/disc data.
The player ZIP needs the user's own extracted ReRAC game data, but not Python.

Changes: safe rail pickups without collection cutscenes; Morph-o-Ray support for
retained/revived enemy lives; horizontal locked aim only in Kalebo test rooms;
physical randomization of Quartu's Bolt Grabber; optional fresh burst-shot aim;
early-O2 Orxon shortcut access while preserving Clank progression; eight-second
fading wrench name/XP feedback on accepted XP (V5 normal cap, V8 NG+ cap).
Detailed Orxon changes: source/docs/mod/ORXON_SHORTCUT_V118.md.
Other v117 changes: source/docs/mod/COMBAT_PICKUPS_V117.md.
Verification limits: source/docs/mod/VERIFICATION.md.

Logs: logs/runtime038-*.log and logs/update-error.log in this kit.
Backups: backup/runtime_038/<timestamp> in your playable folder; restore.json
maps numbered files back to original paths. Keep the window open if an error
occurs and share its diagnostic log. No GitHub operations occur.

This is an independent mod by TomRus47, not part of ReRAC's development.
The mod author owns no original-game rights or assets. Thanks to the ReRAC
team for making this work possible. Upstream licenses and credits apply.
'''

def create(output):
    # Run the exact build preflight before allowing a kit to be distributed.
    validate_source_apis(ROOT)
    tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT)
    names=sorted(set(n.decode() for n in tracked.split(b'\0') if n))
    entries={};hashes={}
    for name in names:
        p=ROOT/name
        if not p.is_file() or p.is_symlink():raise ValueError('Invalid source entry: '+name)
        data=p.read_bytes();entries['source/'+name]=data
        hashes[name]=hashlib.sha256(data).hexdigest()
    entries['source-files.json']=(json.dumps({'patch':'0.1.6.118','runtime_base':'0.3.8-alpha','files':hashes},indent=2)+'\n').encode()
    entries['README_FIRST.txt']=README.encode()
    for name,action in [('01_COMPILE_AND_UPDATE.bat','build'),('02_OPEN_UPDATED_LAUNCHER.bat','open'),('03_CREATE_PLAYABLE_ZIP.bat','package')]:
        entries[name]=batch(action)
    output.mkdir(parents=True,exist_ok=True);archive=output/(NAME+'.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
        for name,data in sorted(entries.items()):z.writestr(NAME+'/'+name,data)
    with zipfile.ZipFile(archive) as z:
        if z.testzip():raise ValueError('Archive integrity failed')
        for name,digest in hashes.items():
            if hashlib.sha256(z.read(NAME+'/source/'+name)).hexdigest()!=digest:raise ValueError('Source digest failed: '+name)
    archive.with_suffix('.zip.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n')
    print(archive);return archive
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    create(parser.parse_args().output)
