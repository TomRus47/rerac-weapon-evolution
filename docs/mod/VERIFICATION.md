# Verification: gameplay 0.1.6.49 / public 0.3.1

## Evidence and scope

The source baseline is the attached v48 update kit reconstructed using its
cumulative developer patch. The two supplied crash records have unsymbolized
stacks. A minimal v48 fixture reproduces the exact pvar panic by dispatching an
old Oltanis arc callback to a replacement empty-pvar slot. The same behavioral
fixture passes with the lifecycle fix. See RESPAWN_LIFECYCLE.md for the limit:
the exact user save has not been run here.

## Local checks

- Rust 1.99.1, Linux x64: `cargo test --release --locked --no-default-features
  -p rc-game -p rc-formats --lib`: **969 game tests passed**, **151 formats tests passed, 1 ignored**.
- Launcher and release tools: `python -m unittest discover -s launcher/tests -v`: **89 passed**.
- Source/default/example/optional embedded-profile integrity, generated reference
  reproducibility and reviewed source hashes: `python tools/mod/check_source.py`.
- Incremental updater transaction, migration, reapply and restore tests, plus an
  exercise using the actual changed source payloads and byte-exact restoration.
- Windows x64 engine type-check uses a disposable source copy with BLAKE3's
  pure implementation enabled only in that copy; no cross-check dependency or
  lockfile changes are shipped. This is not a linked Windows runtime.

The initial unoptimized Linux link failed in this environment with unresolved
compiler-generated closure shims; the normal optimized release build is the
validation target. No production workaround for that local linker issue is
included. Standard release tests must pass normally and in parallel.

## Windows build and remaining manual checks

The GitHub Windows workflow separately builds the linked MSVC runtime, runs
library/launcher tests, verifies exact runtime identity, builds/smoke-tests the
frozen launcher and packages the tracked INI/icon. Its Actions result and
release-build.json, not this local type-check, establish whether a particular
Windows package was built successfully. Draft creation is conditional on success.

Before publishing, test:

- Repeated Oltanis deaths at the reported checkpoint with Tesla, Magneboots and
  Sonic Summoner; no companion, active companion and expired companion.
- Electrical effects after respawn on Oltanis and Quartu; XP, grades, save slots,
  checkpoint location and native encounter state remain correct.
- Pause-menu XP placement on several localized names; ten-drone formations on
  open ground and near scenery; settings/presets restored in the extracted ZIP.
- Experimental shotgun release/cooldown, flame-mine landing and independent
  colors/sound, boot leg poses and steep/downward locked camera.

No game data, Windows executable or active author profile was supplied in this
source kit. Audio probes remain removed. Base-runtime rain/taxi audio is outside
this release's fixed claims. A fresh player ZIP is not equivalent to an in-game
verification of every change.

Historical v40 verification is retained in Git history; it is not a current
claim that audio diagnostics should be enabled or that old test totals apply.
