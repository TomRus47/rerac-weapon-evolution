# Preparing an unpublished release

The previously published descriptor `mod/release.json` and its v0.3 tag are
historical and remain unchanged. `mod/development.json` identifies current code;
`mod/draft.json` identifies the explicitly requested candidate. Its unique
`v0.3-rerac-0.3.6-alpha` identifier separates it from the public v0.3 release
without changing the mod's public version number.

An intentional change to the draft descriptor on main triggers
`.github/workflows/draft-release.yml`. The Windows job checks the tracked source,
runs launcher/game tests, compiles the editable runtime and verifies the frozen
launcher. Packaging uses the approved tracked profile and excludes personal
settings, presets, saves and game data. Existing author-PC binaries are not
silently relabeled as builds of a different commit.

The dependent job validates the player identity report, checksums, default and
icon hashes against the exact checkout. It archives that same commit as source,
then creates a GitHub release using **--draft**. There is no publish option.
An existing public release or a draft for a different commit is refused; an
interrupted same-commit upload can be retried. Every uploaded asset digest and
the private draft state are checked afterwards. A failed build creates no draft.

This workflow does not move the old v0.3 tag or replace its assets. Ordinary
source commits do not create new drafts. To prepare another candidate, review
the source, notes and profile, then deliberately update the descriptor with a
distinct draft tag or remove the superseded draft manually. Publishing is a
separate author action after reviewing the private release page.

`mod/default-profile.json` records the supplied profile's original digest and a
fingerprint of every section/key/value. Comment corrections must not change that
value fingerprint. Default, example, optional embedded profile, generated
reference and source hash index must remain synchronized.
