# Weapon Evolution 0.5.1 source synchronization

## Inputs and scope

- GitHub main baseline: `d337de65344c644da1a8a5b0c995e3562c55c789`.
- Supplied v141 Explosion XP Fix Test and v142 Wrench HUD Recall Test kits.
  The v142 source already includes the complete v141 fix.
- All 1,480 baseline files were verified against GitHub's Git blob hashes
  before integration. Preserve the existing main README/publication work and
  the six release-specific files absent from the local test kits.
- All ten changed runtime, regression-test and runtime-update-tool files are
  byte-identical to v142. Public version/source notes and the hash inventory
  are reconciled separately with main.
- Retain launcher 0.30.2, configuration capability 97, public default/example
  INIs, embedded default, icon, balance, save formats and saved seed generation.

## Verification evidence

The supplied v142 `VERIFICATION.md` records **1,528 rc-game tests passed**, with
three unchanged large historical seed surveys filtered, and **295 Python tests
passed**. Its game fixtures include native explosion attribution across eight
grades and both Gold settings, accepted damage/XP, queued source deletion,
repeated-contact deduplication and wrench feedback/HUD handoff.
These are the kit's earlier results, not a Rust run performed during this sync.

Integration checks passed:

- Full main launcher/build-tool Python suite: **301 passed, 0 failed, 0 skipped**.
  This includes main's six historical 0.5 publication tests.
- Source export: **1,485 tracked files**, **680 indexed mod files** and
  **2,637 default INI values**; all recorded hashes verified.
- Prepared runtime interface/identity and matching candidate descriptor checks.
- Git whitespace checks and byte comparisons of all approved profiles, the
  icon and the original main publication files.

Rust/Cargo are unavailable in the synchronization environment. No new Rust
compilation, Windows linking, real-disc execution or visual playthrough is
claimed. The included Windows build path continues to run native tests, build
the runtime and verify exact 0.1.6.142 identity before installing it.

## Publication boundary

This update changes main source only. The existing release title is 0.5.1 but
its tag remains v0.5 at the earlier v140 commit. Neither that tag nor any
release asset is changed. Automatic release source archives follow the tag;
the current-main archive supplies the synchronized code.
The historical `mod/release-0.5.json`, publication workflow and release notes
are retained. The manual candidate descriptor matches the 0.5.1/v142 source
and remains unpublished.

Weapon Evolution is an independent mod by TomRus47, who develops the mod only,
not ReRAC or the original games. Original-game rights belong to their owners;
ReRAC licenses and contributor credits remain intact. Thanks to the ReRAC
developers and contributors for enabling these mods.
