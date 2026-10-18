#!/usr/bin/env python3
"""Create a shareable mod ZIP from an already compiled editable runtime."""
from __future__ import annotations
import argparse
import sys
from pathlib import Path
import launcher_core as core
from migrate_ini import convert
from build_launcher_exe import build as build_launcher, smoke_test

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path)
    parser.add_argument('--ini', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--no-presets', action='store_true')
    parser.add_argument('--launcher-exe', type=Path, help='Use an existing compiled standalone launcher EXE.')
    args = parser.parse_args()
    try:
        settings = core.read_settings(core.KIT)
        exe = args.exe or core.preferred_exe(core.KIT, settings.get('exe'))
        ini = args.ini
        if ini is None:
            ini = core.bootstrap_ini(core.KIT)[0]
        output = args.output or core.KIT/'dist'/('ReRAC_Weapon_Evolution_'+core.stamp())
        print('Default configuration: '+str(ini), flush=True)
        print('Runtime: '+str(exe), flush=True)
        raw,_ = convert(ini.read_bytes())
        # Fail before downloading/building tools if the game or INI is not ready.
        doc = core.checked_document(raw)
        core.probe_exe(exe, core.required_config(doc))
        if args.launcher_exe:
            launcher_exe = args.launcher_exe.resolve(); smoke_test(launcher_exe)
        else: launcher_exe = build_launcher(core.KIT)
        archive = core.package_distribution(core.KIT, output, raw, exe, not args.no_presets, launcher_exe=launcher_exe)
        print('RELEASE READY: '+str(archive), flush=True)
        print('Includes the standalone launcher EXE, modded game, shaders, INI and presets. Players do not need Python.', flush=True)
        return 0
    except (ValueError, OSError, UnicodeError, core.subprocess.SubprocessError) as error:
        print('ERROR: '+str(error), file=sys.stderr)
        return 2

if __name__ == '__main__': raise SystemExit(main())
