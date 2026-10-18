# Source export verification

Prepared from Weapon Evolution gameplay v0.1.6.39 and launcher v0.5.1 with icon support.

Checks performed:

- 66 Python tests passed: the existing 60 launcher/profile/package tests and six new source-checkout build tests.
- The source map identifies 142 mod implementation/integration files with export hashes.
- The approved public profile contains 888 values. Default and example profiles match, and the optional embedded profile matches the corresponding real-HP session profile.
- The delta applies cleanly to the supplied snapshot after Git-style LF normalization and reproduces all 121 changed/new Rust, Cargo and embedded-profile files covered by that delta.
- Historical backups, generated executables, original source archives, local runtime, saves, personal settings and game data are excluded. Git ignore rules cover generated runtime, build tools, presets, session profiles and local configuration.
- Local Git integrity and a fresh clone of the prepared repository are checked before delivery.

The developer-documentation update adds `DEVELOPER_GUIDE.md`, the generated `CONFIG_REFERENCE.md` and its generator. Its local links, reference reproducibility and Python maintenance recipe are checked separately. The update changes no Rust gameplay or launcher behavior.

The normal editable gameplay code is the v39 code confirmed working by the user. For this source export, the build helper now discovers its own checkout, builds without applying historical source patches and supports a build-only action that needs no game data. Line endings were normalized for Git; the optional fixed-profile INI was synchronized with the English defaults.

Rust/Cargo and Windows are unavailable in the export environment. The Rust tests, full Windows compilation, frozen EXE generation and gameplay have not been rerun on this exported checkout here. Run `01_BUILD_GAME.bat` and test the generated runtime on Windows before publishing a new binary release.

The prepared Git commit records a source export, not reconstructed upstream history. See `mod/provenance.json` and `docs/mod/PUBLISHING.md`.
