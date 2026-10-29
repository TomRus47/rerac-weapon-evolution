# Preparing the unpublished 0.4.1 release

`mod/draft.json` identifies **v0.4.1-rerac-0.3.8-alpha**. The current source is
ReRAC 0.3.8-alpha / mod patch 0.1.6.116 / capability 92 / launcher 0.29.5.
The published 0.4 release and all earlier tags/assets stay unchanged.
`mod/release.json` is retained as historical v0.3 publication metadata; it is
not the identity of this candidate. See [release notes](../releases/v0.4.1.md).

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
