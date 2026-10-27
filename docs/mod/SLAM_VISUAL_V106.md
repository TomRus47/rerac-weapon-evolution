# Wrench blast and upgrade feedback (v106)

Development patch **0.1.6.106**, configuration capability **89**, launcher **0.28.9**, based on **ReRAC 0.3.6-alpha**. Public mod version remains **0.3**. This updates source/main only; no release, tag or published download is replaced.

## Shared collision and visual geometry

The experimental wrench ground slam captures its grade's damage, radius and palette when the attack begins. It triggers once, with the existing five-simulation-tick landing lead. A vertical collision probe finds the surface 0.9 world units ahead of Ratchet; the centre is 0.05 units above that surface. Collision-less test/tool sinks retain their documented ground-height fallback.

`hero/melee.rs` passes that exact centre and configured `slam_explosion_radius` to `World::ground_blast_targets`. This uses the ordinary moby collision primitives with clipped grid traversal, allowing a ground-centred sphere to cross world z=0. It does not move/enlarge the sphere or change other native queries. Accepted per-instance HP loss belongs to the wrench; nearby destructible objects retain the native bomb hit message. A second callback for the same attack cannot repeat the explosion.

The old cosmetic bomb helper emitted independently travelling fireballs, rings and flashes. Its size parameter was not an outer radius, so those visuals could suggest a larger damage area. The slam now snapshots a render-only `SlamVisual` from the **same centre and radius** used by collision. `upgrade_wave_render.rs` draws a filled, translucent sphere that reaches that radius in six ticks and fades in place over 24 ticks. Its vertices never expand beyond the damage sphere. Enemy bodies intersecting the sphere can take damage even when their actor origin is outside it; the visual represents the query volume, not a centre-distance test.

The independent per-grade palette remains configurable; `default` uses a warm orange. No new configuration, save fields, lights or explosion actors are introduced. Damage, XP, sound, centre and once-per-attack identity are unchanged by the visual correction. Weapon-upgrade bursts keep their own expansion and grade colors.

## Included fixes from local tests v103–v105

- Restore accepted-hit feedback at the normal scheduler's post-update boundary. Ignite immediately on accepted fire hits, so a one-hit victim can still show the finite burn batch. Repeated contact refreshes the effect without stacking a new particle batch every frame.
- Keep primitive geometry, fine-coordinate indices and additive indices together when inserting the weapon XP HUD batch. `weapon_bar_behind_hud` chooses stacking independently from `weapon_bar_alternative_position`. Both controls are visible in the XP Bars group and localized in all five launcher languages.
- Resolve slam damage through native target collision and real-HP instance handling, with correct ground height, object messages and wrench XP.
- Use the existing filled upgrade renderer for an XP-earned maximum-Nanotech increase: one bright sky-blue **#00BFFF** burst. Loading, death, counter rewind and restored progression do not replay it. Premium/Ultra purchases and ordinary healing do not trigger this XP effect.

## Local Windows update

The v106 delta kit is for an **already installed v105** source/playable pair. Extract it into a separate folder and run `01_UPDATE_BUILD_AND_OPEN.bat`. Select the existing Rust checkout and the playable folder containing your current launcher EXE, `weapons.ini` and `runtime/rerac.exe`.

Exact-block preflight preserves unrelated source changes and rejects unrecognized inputs before writing. The updater backs up changed source and runtime, runs the rc-game library tests and native Windows release build, verifies editable runtime identity/capability, then replaces only `runtime/rerac.exe`. A failure restores source and the previous playable runtime. The launcher stays **0.28.9**; personal INIs, presets, saves and icon are retained. The test updater does not contact GitHub.

`02_CREATE_PLAYABLE_ZIP.bat` packages the verified installation with the current INI and existing launcher. It creates a ZIP/checksum without presets, settings, saves or game assets. `tools/mod/create_slam_visual_test_kit.py` recreates the source delta kit from this checkout.

For a fresh full source checkout, use the regular root build scripts instead of applying an incremental patch. See [build layout](BUILD_LAYOUT.md) and [verification](VERIFICATION.md).

## Ownership and credits

Weapon Evolution is an independent mod by TomRus47. The mod author is not a developer of ReRAC or the original games and claims no rights to the original games or their assets. ReRAC and original-game credits/licenses remain applicable; see [credits](CREDITS.md). Thanks to the ReRAC developers for the port and the work that makes these modifications possible.
