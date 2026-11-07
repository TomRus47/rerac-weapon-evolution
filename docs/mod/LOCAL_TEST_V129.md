# Weapon Evolution v129 — quick-select audio and underwater surfaces

This is a **local test** of Weapon Evolution on **ReRAC 0.3.8-alpha**. No GitHub
operations or release publication are performed. Launcher version 0.29.7 and
configuration capability 94 are retained; no INI key, default value or save
format changes.

## Compile and install

1. Extract this ZIP into a new folder. Close the game and mod launcher.
2. Run **01_COMPILE_AND_UPDATE.bat** from this new folder.
3. Select your **existing PLAYABLE MOD folder** containing
   `ReRAC_Weapon_Evolution.exe`, `weapons.ini` and `runtime/rerac.exe`.
   This is the only folder selection. Do not select `source` or an old update kit.
4. Let the tests, build and runtime identity checks finish. The runtime and
   matching shaders are installed transactionally, with a backup of prior files.
5. Open the same installed launcher, or use **02_OPEN_UPDATED_LAUNCHER.bat**.

The complete reviewed source is bundled. The v125/v126/v128 update folders are
not needed. Existing launcher EXE/icon, INIs, presets, saves, randomizer seeds,
XP, balance and quick-select assignments are retained. No new game is required.
The preceding walloper, skill-point, dual-ring, pause, water-blending and
Thruster-Pack preloading changes are included.

**Launch normally rather than with the earlier water diagnostic BATs.** Close
any diagnostic game/launcher before opening the normal launcher.

The build reuses an explicitly marked short private Rust cache from the v126.1
recovery when present. Otherwise its existing fallback uses a short per-user
cache. Rust must compile, link and execute a small test before Cargo cleans
local workspace artifacts. The global Rust installation, security settings and
permanent environment are not modified.

After successful installation, **03_CREATE_PLAYABLE_ZIP.bat** packages the
installed runtime/launcher with the current INI. It produces a local ZIP only.

## Immediate audio while quick select freezes the world

The v125 pause stopped gameplay EE sound updates. The audio renderer kept
producing IOP/SPU samples, but R2 ring changes and R1 experimental-mode changes
only allocated an EE-side logical sound slot. Its start command did not reach
the mixer until gameplay resumed.

`AudioSystem::quick_select_frame` now runs the sound-slot/music part of the EE
audio update on the **private audio RNG**, without level-start, sound-instance
emitters, gameplay ticks or reverb-volume tests. `audio_out::run_audio` invokes
it only for sample frames not already produced by a game/menu/scene update,
while quick select is frozen (and its closing frame). The existing output clock
still produces exactly one 800-sample frame per 60 Hz audio frame. It never
renders an extra frame just because a selection changed.

The world clock, hero, enemies, projectiles and particles remain frozen. Ring
selection, per-weapon R1 limits, remembered ring and per-save state are unchanged.
Existing audio behaviour with quick-select pause disabled is retained.

Test with `Pause on quick select` On: hold Triangle, change rings with R2 and
change a supported weapon mode with R1. Each confirmation should sound while
Triangle is still held. On releasing Triangle, old selection sounds should not
play again. Repeat with the pause setting Off and in the pause-menu editor.
No audio recording or trace probe is enabled by this update.

## Remaining Pokitaru purple surface

Diagnostic `RC_FOG=0` removed the purple while retaining the underwater state,
fullscreen tint and ocean. The v128 exemption for ocean class 1111 reduced the
problem but did not remove it. Pokitaru also has separate pool overlays
(1903/1919) and patch-water/reflection materials; v128 still applied the world's
short-range underwater fog independently to these layers.

The shared `fog_state::liquid_surface_fog` policy now covers **all liquid surface
draw groups** and strip/ripple water materials while the camera is submerged.
It changes only a copy of their material fog-enable flag. The scene's world fog,
enemy/particle fog, fullscreen underwater tint, water heights, collisions,
transparency, blend equations and surface geometry are retained. Above-water
surface fog is restored from the unchanged scene fog as soon as the camera
emerges. The water and sea draws follow the current camera's FogSet update.
Fire fields share no exemption and retain their normal fog.

This is a targeted continuation of the confirmed fog A/B test, not an in-game
visual verification. Dive at the same Pokitaru location, look up at the sea,
then surface. Check that the remaining purple patches disappear and underwater
visibility/colour still look reasonable. If any purple remains, supply a normal
launch screenshot of that view; do not disable global fog as a permanent fix.

## Source and checks

The supplied v128 source archive was imported only after all **1,424 source-file
hashes** matched its inventory. It includes the actual dual-ring/pause code and
the previous local fixes. No older source base is used for this export.

- A synthetic audio regression reproduces the IOP-only failure, then starts
  confirmation sounds with no game frame, no extra samples and no duplicate
  start on the following update.
- The liquid-fog test checks underwater exemption, preservation of scene fog,
  above-water behaviour and an existing `RC_FOG=0` configuration.
- Build-tool tests cover reuse of marked short Rust caches, rejection of
  unmarked directories, full compiler link/run probes and runtime-only updates.
- Full source and archive inventories are checked before delivery and again
  by the Windows installer. Existing INIs/icons and the launcher UI source are
  verified unchanged.

The native Windows build and game graphics/audio behaviour still require the
local test. Engine checks on the development host do not substitute for that.

This independent mod is created by TomRus47, who is not affiliated with ReRAC's
development or the original games' developers. Original game content belongs
to its respective rights holders. Thanks to the ReRAC developers for their work
on the runtime that makes this mod possible.
