# Weapon Evolution v128 — underwater ocean and Thruster-Pack first-use test

Local test for **ReRAC 0.3.8-alpha**, continuing from the verified v126 kit.
No GitHub operations or release publication are performed.

## Install and test

1. Keep the original v126 update folder, including `work` and `last-build.json`.
   Install v126 successfully first; its v126.1 build recovery remains valid.
2. Extract this new kit to its own folder. Close the game and mod launcher.
3. Run **01_COMPILE_AND_UPDATE.bat**.
4. First select the **existing v126 UPDATE folder**, for example
   `C:\Users\USERNAME\Desktop\ReRAC_Weapon_Evolution_v126_Walloper_Water_Skill_Points_Test`.
   This first selection is neither the playable mod nor the v125 kit.
5. For the second selection, accept/select the **playable mod folder** containing
   `ReRAC_Weapon_Evolution.exe`, `weapons.ini` and `runtime/rerac.exe`.
6. Wait for the build and identity checks. The existing launcher EXE/icon is kept.
   Use **02_OPEN_UPDATED_LAUNCHER.bat** or open your usual installed launcher.

**Launch normally, not through the earlier water diagnostic BATs.** The fix is
intended to work with normal fog enabled. Close any diagnostic launcher first.

No new game is needed. Existing INI values, presets, seeds, saves, XP, balance,
dual quick-select rings and quick-select pause setting are retained. No new
configuration key or save-format change is introduced. Only a private copy of
the supplied v126 source is patched and compiled.

The previous downloaded private Rust compiler is reused through the short cache
created by the v126.1 recovery, or copied there if necessary. A full compile,
link and execution of a small Rust test now runs before Cargo cleans its local
workspace packages. A required fallback download also uses a short per-user
path. The global Rust installation and permanent environment are untouched.

**03_CREATE_PLAYABLE_ZIP.bat** packages the installed launcher/runtime and current
INI after a successful build. This creates a local ZIP; it does not upload it.

## Underwater ocean

The reported Pokitaru purple surface disappeared in diagnostic test 01
(`RC_FOG=0`), while the automatic underwater state, fullscreen tint and ocean
remained enabled. That identifies fog as a necessary part of the reported
rendering problem; it does not establish a driver-independent reproduction.

The ocean is a separate callback from the water strips/ripples corrected in
v126. Its surface contains additive, already distance-coloured passes. Applying
the world's short underwater colour fog to those passes accumulates that colour
again per layer. This test exempts **only the ocean callback while the camera is
underwater** from that additional material fog. Above-water ocean drawing is
unchanged. World geometry, enemies, other liquids and particles retain fog, and
the fullscreen underwater tint remains enabled. Ocean drawing is explicitly
scheduled after the current camera/fog update so surface crossings use the same
underwater state. Camera collision, water levels and swimming physics are unchanged.

Validation target: at the same Pokitaru location, dive through the sea line, look
up, then surface again. Check that the purple ceiling is gone, normal underwater
colour/visibility remains, and the above-water horizon still looks correct.
This visual result must be confirmed in game.

## Thruster-Pack first-use stall

The first flame draw for a newly visited planet previously opened two overlays,
parsed them and relocated the flame tables synchronously on the render thread.
This work is now done by the existing level-loader worker using its explicit
destination index and shared parsed overlays, before its recognition caches are
released. The render callback performs a read-only cache lookup: no file read,
overlay parsing or table relocation can start there. Preparing another planet
does not change the active-level global, game RNG, flame animation or GPU state.

This removes a concrete source of first-use frame stalls. It does **not** claim
that every possible shader compilation, texture upload or driver stall is gone.
Flame geometry, particle counts, colour, sound and gameplay are unchanged.

Validation target: start the game normally, land on a planet with the
Thruster-Pack equipped, use its jump/boost immediately, repeat, then change
planet and try again. Report whether the first-use hitch persists, and whether
subsequent uses also hitch. Keep the normal launcher session log if it persists.

## Verification and scope

The source includes regression checks for underwater ocean fog isolation and
cache lookup behaviour, plus installer/build-tool tests. Exact-context patching
and full v126 inventory verification run before any private source is prepared;
unknown or edited inputs are rejected. The installed runtime/shaders are updated
only by the original identity-checked transactional builder after success.

No game data, saves, Windows executable or personal paths are included in this
source patch. In-game graphics/performance confirmation is still required.

This independent mod is created by TomRus47, who is not affiliated with ReRAC's
development or the original games' developers. Original game content belongs
to its respective rights holders. Thanks to the ReRAC developers for the runtime
work that makes these mods possible.
