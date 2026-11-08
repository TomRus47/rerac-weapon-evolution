# Publishing Weapon Evolution 0.5

`mod/release-0.5.json` records the author's explicit publication request, matching
public/internal versions and the download policy. Gameplay remains v140 on
ReRAC 0.3.8-alpha; this release changes packaging/documentation, not balance.
The approved default profile and icon retain their existing fingerprints.

The dedicated workflow is triggered by a change to that descriptor on main, or
manual dispatch. The Windows job checks the tracked export, all Python tests,
format/game tests, the native engine, frozen GUI, INI and icon. Its artifacts
retain the verification report/checksum for maintenance. They are not release
attachments. Failed tests/builds cannot enter the publishing job.

The publisher is scoped to this repository, the exact workflow commit and tag
`v0.5`. It verifies a private source archive, checks/creates a matching immutable
tag, uploads only the Windows ZIP to a draft, validates the uploaded SHA256 and
exact one-asset list, and then publishes the approved release as latest.
It never moves an existing tag or modifies an already public release.

GitHub supplies native Source code ZIP and TAR.GZ from that same tag. These
cannot be confused with checksum/JSON attachments because no technical sidecars
or duplicate custom source ZIP are uploaded. In the Windows package,
`README_FIRST.txt`, `CHANGES_0.5.md` and `BUILD_INFO.json` contain instructions,
release notes and provenance. The handoff/GUI/gameplay checks retain their scope;
no extra in-game playthrough is claimed by the publisher.

`mod/release.json` and its legacy publisher retain the older v0.3 authorization
record. They are not used by the 0.5 publisher. `mod/draft.json` remains a separate
manual candidate descriptor. Its workflow does not run on normal development
pushes. Older release assets and tags are preserved.

Weapon Evolution is an independent mod by TomRus47, not ReRAC or original-game
development. Original-game rights, upstream licenses and contributor attribution
remain with their respective owners; ReRAC's work makes the mod possible.
