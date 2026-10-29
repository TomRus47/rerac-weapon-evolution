# v115 local verification — 2026-10-28

No GitHub repository, draft, release, tag or remote asset was changed.
This small overlay is intended for an extracted v114 source kit; v113 is also
accepted. The old source checkout and installed player configuration are not
patched in place. Existing Gold Bolt and space-demo completion source files are
explicitly protected from replacement.

Completed checks on the available v113 full-source baseline:

- All 1,385 supplied input source hashes verified before editing.
- **1,372 rc-game library tests passed**, release profile with locked dependencies
  and no default features. The unchanged thousand-layout randomizer survey was
  excluded on this host; no seed generation or routing code was changed.
- **198 Python launcher/installer/packaging tests passed**, including rollback
  of the combined runtime/GUI update, old-profile compatibility, the expanded
  bonus range and save-disabled local test environment.
- **6 source-overlay tests passed**: both supported version identities, preserved
  v114 pickup canaries, original-source immutability, corrupt input/payload,
  unknown local changes, protected paths, symlinks and directory/version errors.
- `cargo check --release --locked --no-default-features -p rc-engine` passed,
  also with `--tests`, checking production and test code against ReRAC 0.3.8.
  This Linux check used a host-only pkg-config stub for unavailable native SDKs.
  That stub is outside the source and is not distributed. This is a typecheck,
  not native linking, a Windows EXE build or a GPU/gameplay test.
- The complete prepared source inventory and both protected pickup source hashes
  were checked again after overlay preparation. Existing default INIs were not
  modified. Native Windows installation performs its own Rust tests, real build,
  runtime identity check and standalone GUI smoke test before replacing files.

New regressions cover each current/max HP pair through 300; capacity conservation;
51 max HP becoming purple at 2 HP and blue at 1 HP; all color boundaries; original
alpha/highlight preservation; additive native upgrades and the final 300 limit;
save round trips above 255; upgrade notices/bursts above 255; disposable Veldin II
startup; normal/gold Morph-o-Ray conversions, allocation failure, retained source
actors, duplicate callbacks, XP-disabled settings and reused dynamic slots.

In-game validation still requires the player's own extracted data and Windows
runtime. Use `04_TEST_NANOTECH_VELDIN_II.bat` after installation. The test does not
read or write ordinary save slots.

Weapon Evolution is an independent mod by TomRus47. The author is not a ReRAC
developer and owns no original-game rights or assets. Existing licenses and
credits apply. Thanks to the ReRAC developers for making these modifications
possible.
