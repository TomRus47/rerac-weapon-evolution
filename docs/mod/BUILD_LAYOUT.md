# Source, runtime and player package layout

## Ready-to-play Windows download

`ReRAC_Weapon_Evolution_0.3_windows_x64.zip` contains:

```text
ReRAC_Weapon_Evolution_0.3_windows_x64/
  ReRAC_Weapon_Evolution.exe
  weapons.ini
  weapons.example.ini
  launcher.ico
  launcher-settings.json
  runtime/
    rerac.exe
    build-info.json
    assets/shaders/
  licenses/
```

Extract the entire ZIP and open the mod launcher EXE. Keep the runtime and
configuration beside it. Players supply extracted disc data, but do not install
Python, Rust or compiler tools. The launcher automatically selects its included
modified runtime and can discover `%LOCALAPPDATA%\rerac\games\rac1\data`.
Read the [credits and rights notice](CREDITS.md).

## Source download

`ReRAC_Weapon_Evolution_0.3_source.zip` contains Cargo.toml, crates/, launcher/,
tools/ and documentation from the exact Windows build commit. No compiled game
is committed or included in that source ZIP.

In an x64 Visual Studio developer prompt, run `01_BUILD_GAME.bat` at the source
root. Its pipeline is:

1. Prepare/migrate `launcher/weapons.ini` without replacing existing values.
2. Validate the source root and run Rust game-library tests.
3. Build `rc-engine` into `target/release/rerac.exe`.
4. Run `--version-json` to check the modded editable runtime.
5. Stage the executable, shaders and identity into `launcher/runtime/`.

A passed Rust test suite alone does not create runtime/. The engine build and
staging must also finish successfully. If compilation fails, the previous staged
runtime is preserved. The root build BAT does not prompt for a destination:
it uses that checkout's launcher/ folder. `02_OPEN_LAUNCHER.bat` opens its editor.

`BUILD_FINAL_WINDOWS_EXE.bat` then packages a complete player download. It is
separate from merely compiling the game or opening the Python editor.

## Local overlay/test ZIP

The optional local test kit has a different layout: source/ plus updater BATs.
`01_UPDATE_BUILD_AND_OPEN.bat` asks for two independent folders:

- ReRAC source checkout: contains Cargo.toml, Cargo.lock and crates/.
- Mod launcher destination: contains or receives weapons.ini, Python editor
  files and runtime/. It must be separate from the source checkout.

Compilation reuses the chosen source checkout's target/. The staged game goes
into the **chosen mod destination**, not automatically into the new update ZIP.
A folder named v57 can therefore contain the current v59 runtime after updating.
The runtime's --version-json identity determines its version, not the folder name.
The kit records source/mod paths in local-test-settings.json beside its BATs.

`00_RESTORE_LAUNCHER.bat` only replaces Python editor code with backups. It also
accepts source/launcher directories without a compiled game and never creates a
runtime. `02_OPEN_TEST_LAUNCHER.bat` reopens the selected editor. The standalone
launcher EXE is not rebuilt by this fast restoration path.

## Diagnostics

Run compilation in an existing developer prompt to keep the final error visible.
Check target/release/rerac.exe first, then launcher/runtime/rerac.exe for a direct
source build, or the selected destination's runtime/rerac.exe for an overlay build.
If the first is missing, the engine build did not complete; if only staging is
missing, inspect --version-json, shaders and the final copying/rename error.
