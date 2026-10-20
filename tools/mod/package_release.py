#!/usr/bin/env python3
"""Package a freshly built Windows runtime and record exact release identities.

Run after launcher/editabile.py build. This does not read author presets/settings
or compile gameplay. The normal packager builds and smoke-tests the launcher,
including its Tcl/Tk resources and supplied custom icon, before producing a ZIP.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
KIT = ROOT / "launcher"


def main() -> int:
    if os.name != "nt":
        raise ValueError("Build the Windows player release on Windows x64.")
    release = json.loads((ROOT / "mod/release.json").read_text(encoding="utf-8"))
    exe = KIT / "runtime/rerac.exe"
    identity = json.loads(subprocess.check_output(
        [str(exe), "--version-json"], text=True, encoding="utf-8", timeout=20
    ))
    if (identity.get("weapon_evolution_patch") != release["patch"]
            or identity.get("weapon_evolution_config") != release["config_capability"]
            or identity.get("weapon_evolution_audio_diagnostics") is not False
            or str(identity.get("version", "")).endswith("-fixed")):
        raise ValueError("The staged runtime is not the clean editable release build.")
    destination = KIT / "dist" / release["asset"]
    subprocess.run([
        sys.executable, str(KIT / "package_launcher.py"),
        "--exe", str(exe), "--ini", str(KIT / "weapons.default.ini"),
        "--output", str(destination), "--no-presets",
    ], cwd=ROOT, check=True)
    archive = Path(str(destination) + ".zip")
    report = {
        "release": release["tag"], "runtime": identity,
        "launcher": release["launcher"],
        "commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "archive": archive.name,
        "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
        "default_ini_sha256": hashlib.sha256((KIT / "weapons.default.ini").read_bytes()).hexdigest(),
        "icon_sha256": hashlib.sha256((KIT / "launcher.ico").read_bytes()).hexdigest(),
        "gameplay_validation": "Pending; keep the GitHub release as a draft.",
    }
    (destination.parent / "release-build.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    print("Player package verified: " + archive.name, flush=True)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print("ERROR: " + str(error), file=sys.stderr)
        raise SystemExit(1)
