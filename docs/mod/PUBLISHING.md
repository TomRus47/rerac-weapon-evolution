# Windows packages and GitHub releases

This is the full source repository for an independent mod of ReRAC 0.3.4-alpha.
Keep the mod author, ReRAC developers and original game's developers distinct.
Preserve the ISC license and existing copyright notices. Do not upload disc data,
saves, personal settings or build folders as source commits.

## Build and review locally

Use Rust 1.99.1 (MSVC), Visual Studio C++/Windows SDK and x64 Python with Tk.
Run `01_BUILD_GAME.bat` in an x64 developer prompt. The game is built from this
checkout and staged with shaders in `launcher/runtime/`.
Verify `launcher/runtime/rerac.exe --version-json`: patch 0.1.6.56, capability 56,
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

## Automated Windows build and release

`.github/workflows/windows-release.yml` runs when `mod/release.json` changes on
main, or from Actions > Windows player package and release > Run workflow.
It uses a Windows x64 runner, pinned action revisions, Rust 1.99.1 and Python 3.12.
Sources, launcher tests, formats tests and game tests must pass before packaging.
The build job has read-only repository permission. The final draft job alone has
contents-write permission and uses GitHub's short-lived workflow token.

On success it uploads an Actions artifact containing the player ZIP, checksum
and `release-build.json`. Ordinary pushes create **test artifacts only**. The exact commit message
`Release Weapon Evolution 0.3` is the author-approved opt-in for this release:
after the Windows build passes, it creates a draft with all assets and publishes
it as the latest release. The publication step rejects every tag except `v0.3`.
This authorization does not apply to future versions. To create a draft later, explicitly run the workflow
manually and enable `create_draft` (off by default). That opt-in creates a
**draft** with the committed English notes plus an exact-commit source ZIP
and SHA256. An existing release/tag is preserved, never overwritten or
published by an ordinary build. If creation is unavailable because of repository
policy, download the artifact and attach it manually to a new release.

Review the runtime identity, checksum, release notes and in-game results before
publishing. Tests/build smoke checks cannot establish visual behavior or confirm
an unsymbolized crash report's exact save path. Only the exact author-approved 0.3 release commit publishes automatically.

## Versioning and source downloads

Public tag `v0.3` follows the published `v0.2` release. It identifies the mod,
not a new ReRAC version: compatibility remains ReRAC 0.3.4-alpha. Gameplay patch
0.1.6.56, launcher 0.16.0 and profile capability 56 have separate purposes.
Update `mod/release.json`, runtime identity and documented checks deliberately.
For a future profile feature also update capability negotiation and all grades.

Use [docs/releases/v0.3.md](../releases/v0.3.md) as the release body. Attach
the player ZIP, its SHA256 and identity report as assets. To attach an explicit
developer download, run Actions > Attach matching release source with the
existing release tag. It reads the Windows package's `release-build.json`,
checks its ZIP digest, validates that exact source commit, and attaches
`ReRAC_Weapon_Evolution_<version>_source.zip` and its SHA256. The ZIP contains
the complete tracked project, launcher, configuration examples and developer
documentation, without disc data or compiled EXEs. Existing assets and the
release's draft/published state are preserved; uploads never replace assets.

GitHub also adds its usual Source code ZIP/tar downloads when a tag exists.
The explicitly named source asset remains tied to the compiled Windows
package even if the default branch later receives more changes.

Local INIs/presets/settings are ignored. The supplied launcher icon is tracked.
The original export tags remain historical; do not rewrite published commits.
