# Windows packages and GitHub releases

Weapon Evolution is an independent mod for ReRAC 0.3.4-alpha. TomRus47 develops
this mod only, does not develop the original games or ReRAC, and owns no rights
to the original games or assets. Keep the [credits](CREDITS.md), ISC license and
existing contributor notices intact. No disc data, saves or author settings ship.

## Local build

Use Rust 1.99.1 MSVC, Visual Studio C++/Windows SDK and x64 Python with Tk.
`01_BUILD_GAME.bat` tests/builds this checkout and stages launcher/runtime/.
Check --version-json: patch 0.1.6.59, capability 58, editable mode and audio
diagnostics false. [BUILD_LAYOUT.md](BUILD_LAYOUT.md) distinguishes source,
local overlay and ready-to-play packages.

`python tools/mod/package_release.py` packages the tracked English default and
custom icon, builds/smoke-tests the frozen launcher, and records exact commit,
runtime, INI, icon and ZIP identities in release-build.json. Personal presets and
settings are excluded. To package a custom profile instead, explicitly pass it
to launcher/package_launcher.py. Share the complete player ZIP, not one EXE.

## Windows workflow

.github/workflows/windows-release.yml builds on changed mod/release.json on main,
or on manual dispatch. Pinned actions run Python, format and game-library tests,
link the Windows engine, build/smoke-test the launcher, and upload player assets.
The build job has contents-read permission; only the release job can write assets.
A failed build cannot replace the existing release.

Ordinary pushes produce artifacts only. The explicit commit message
`Release Weapon Evolution 0.3` permits creation/publication of v0.3 after a passed
build. `Update Weapon Evolution 0.3` permits the author-requested replacement of
that existing release, also only after a passed build. Manual create_draft stays
off by default and never authorizes replacement. These opt-ins do not authorize
publishing another public version.

`tools/mod/publish_release.py` validates the player ZIP, sidecar and identity
against the current source. It archives that exact commit as the named source ZIP
and verifies every tracked member. For replacement, it checks the old tag SHA
recorded in mod/release.json before uploading; uploads with --clobber only the
named matching player/source files and identity report; verifies GitHub's asset
digests; rechecks the tag; then moves v0.3 to the new source commit and updates
the existing release title/notes. The release URL and version stay the same.
Do not rewrite existing commits. Future replacement needs a freshly reviewed tag
SHA and explicit author authorization, rather than reusing an old tag lease.

GitHub's automatic Source code ZIP/tar links follow the updated tag. The named
source ZIP is also attached with its own SHA256 and always corresponds to the
compiled Windows package. Source code is the full modified project, not only a
patch. .github/workflows/source-release.yml can attach matching source for an
existing build that lacks it; it preserves existing assets rather than clobbering.

Read release-build.json and the successful Actions run for actual Windows build
evidence. Smoke checks and unit tests cannot establish every disc-specific
visual, collision or saved-game scenario. Preserve the manual checklists in the
implementation notes and the author-approved scope in the release description.
