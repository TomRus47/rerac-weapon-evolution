# Windows packages and GitHub releases

This is the full source repository for an independent mod of ReRAC 0.3.4-alpha.
Keep the mod author, ReRAC developers and original game's developers distinct.
Preserve the ISC license and existing copyright notices. Do not upload disc data,
saves, personal settings or build folders as source commits.

## Build and review locally

Use Rust 1.99.1 (MSVC), Visual Studio C++/Windows SDK and x64 Python with Tk.
Run `01_BUILD_GAME.bat` in an x64 developer prompt. The game is built from this
checkout and staged with shaders in `launcher/runtime/`.
Verify `launcher/runtime/rerac.exe --version-json`: patch 0.1.6.49, capability 48,
audio diagnostics false, and an editable version without the `-fixed` suffix.

Run the gameplay cases in [VERIFICATION.md](VERIFICATION.md), especially repeated
death reloads on Oltanis. Then package either the tracked release default with
`python tools/mod/package_release.py`, or an edited local profile explicitly:

```bat
py -3 launcher\package_launcher.py --exe launcher\runtime\rerac.exe --ini launcher\weapons.ini --no-presets
```

The launcher builder verifies its x64 EXE, bundled Tcl/Tk, startup/self-test and
custom icon cache. The ZIP includes the launcher, modified game, shaders, INI,
icon and licenses. Share the complete generated ZIP and SHA256 sidecar. A
launcher-only EXE update cannot incorporate a gameplay correction into an old
runtime. No official ReRAC-launcher build import is needed for this standalone
player package.

## Automated Windows draft

`.github/workflows/windows-release.yml` runs when `mod/release.json` changes on
main, or from Actions > Windows player package and draft release > Run workflow.
It uses a Windows x64 runner, pinned action revisions, Rust 1.99.1 and Python 3.12.
Sources, launcher tests, formats tests and game tests must pass before packaging.
The build job has read-only repository permission. The final draft job alone has
contents-write permission and uses GitHub's short-lived workflow token.

On success it uploads an Actions artifact containing the player ZIP, checksum
and `release-build.json`, then creates a **draft** release with the committed
English notes. An existing release/tag is preserved, never overwritten or
published by this workflow. If creation is unavailable because of repository
policy, download the artifact and attach it manually to a new release.

Review the runtime identity, checksum, release notes and in-game results before
publishing. Tests/build smoke checks cannot establish visual behavior or confirm
an unsymbolized crash report's exact save path. The workflow never publishes a
release automatically.

## Versioning and source downloads

Public tag v0.3.1 follows the existing public v0.3 release. It identifies the mod,
not a new ReRAC version: compatibility remains ReRAC 0.3.4-alpha. Gameplay patch
0.1.6.49, launcher 0.10.0 and profile capability 48 have separate purposes.
Update `mod/release.json`, runtime identity and documented checks deliberately.
For a future profile feature also update capability negotiation and all grades.

Use [docs/releases/v0.3.1.md](../releases/v0.3.1.md) as the release body. Attach
the player ZIP, its SHA256 and identity report as assets. GitHub automatically
adds Source code ZIP/tar downloads for the tag; those contain the full source
and no EXEs. A separate upload of the same source archive is unnecessary.

Local INIs/presets/settings are ignored. The supplied launcher icon is tracked.
The original export tags remain historical; do not rewrite published commits.
