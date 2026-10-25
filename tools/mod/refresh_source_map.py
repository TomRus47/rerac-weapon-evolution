#!/usr/bin/env python3
"""Refresh reviewed mod hashes; use --add for new implementation/integration files.

This updates the index only. It does not change gameplay or synchronize INIs.
Review the diff before committing: hashes acknowledge the current file bytes.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--add", action="append", default=[], metavar="PATH")
    parser.add_argument("--remove", action="append", default=[], metavar="PATH")
    args = parser.parse_args()
    index = ROOT / "mod/source-map.json"
    record = json.loads(index.read_text(encoding="utf-8"))
    rows = {row["path"]: row for row in record["files"]}
    for name in args.remove:
        rows.pop(name, None)
    for name in args.add:
        path = (ROOT / name).resolve()
        if not path.is_relative_to(ROOT) or not path.is_file():
            raise ValueError("Expected a file inside this checkout: " + name)
        name = path.relative_to(ROOT).as_posix()
        rows.setdefault(name, {"path": name, "reasons": ["Weapon Evolution implementation or integration"]})
    for name, row in rows.items():
        data = (ROOT / name).read_bytes()
        if Path(name).suffix == ".bat":
            data = data.replace(b"\r\n", b"\n")
        row["sha256"] = hashlib.sha256(data).hexdigest()
    release = json.loads((ROOT / "mod/development.json").read_text(encoding="utf-8"))
    record.update(gameplay_version=release["patch"], launcher_version=release["launcher"],
                  files=[rows[name] for name in sorted(rows)])
    index.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"Refreshed {len(rows)} reviewed source hashes.")


if __name__ == "__main__":
    main()
