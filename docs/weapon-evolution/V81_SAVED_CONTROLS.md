# Saved controls and manual weapon modes

This change targets the v80 Weapon Evolution source on ReRAC 0.3.6-alpha. It does not change upstream button constants, original menu tables, native save layouts, damage events or ammunition costs.

## Ownership of state

`weapon_evo_controls.rs` contains the physical-input policy, supported-mode catalog, binding validation and normal inventory requests. `Evolution.control_settings` is configuration loaded from `[general]`; `save::Progress.controls` is player-owned state. `Evolution::set_controls` publishes effective mode switches without clearing combat epochs, XP or damage event queues. Existing `sync_progress`/`restore_progress` paths carry controls through death, travel, load and save. Menu assignments update the persistent GameState; a radial mode change updates both persistent state and Evolution immediately.

The two global flags are opt-in. Their launcher fields are appended at the bottom of the first base card, outside encounter and Ammo-Survival settings. Labels and help exist in English, Italian, French, German and Spanish. Personal configuration migration adds missing false defaults; it does not replace balance values or preset files.

## Physical inputs and native semantics

`PadState.physical_buttons` stores current/previous unmirrored digital buttons before analog-direction merging and gameplay locks. The native pad representation is otherwise unchanged. `physical_edges` prefers this snapshot, with a raw/previous-raw fallback for synthetic pads. This matters because the radial sets lock 2, masking the gameplay `raw`, and the native `raw_pressed` compares against stick-merged previous input. Reading either directly would miss a legitimate R1 or D-pad press.

Only a clone passed to direct Ratchet hero/item updates is normalized. Physical R2 aliases Circle, or Square while the wrench is equipped. Physical L1 aliases the native R1 crouch semantic; physical R1 and R2 are removed from crouch input. Holding both fire buttons is a logical union, including press/release edges and pressure, so releasing one does not stop the other. L2 is consumed by active combat-camera lock. L1 no longer requests first person while this control layout is active. Camera and menu systems retain physical inputs.

A shared allowlist excludes alternate bodies, scripted holds, vehicles, turrets and death. Weapon extraction may block a shortcut but must not switch the fire/crouch mapping midway through a held button. D-pad mode removes only digital movement directions from gameplay; analog axes and flick history remain available. Menu assignment uses only one physical directional edge; simultaneous diagonal assignments are ignored.

Bindings request the native hand/feet/head/back inventory slots. Ownership is `global.owned`, not `global.acquired`, because the latter can reset in NG+. The existing Drone Device request flag is used. No binding grants items, ammunition, XP or a usable state that the native equipment rules forbid.

## Pause and radial rendering

The existing Weapons and Gadgets grid callback handles L1+direction before cursor navigation, so neither the stick nor digital direction moves focus during assignment. Non-item/unowned/non-equippable cells are rejected. An item can occupy one direction; assigning it elsewhere clears the previous direction. Four one-pixel strips form a persistent unfilled border two pixels beyond the native selected-icon border. Colors are stored in the game's GS BBGGRR convention.

The Weapons-only assignment hint identifies the page by its Ammo widget rather than a translated title or fixed page address. It centers in the existing title/grid gap, outside panel render targets, with native L1 glyph 0x14, an outlined D-pad silhouette and a translated label. Gadgets has no hint.

The optional `QuickSelect::draw_with_controls` keeps the original draw API available. It emits native R1 glyph 0x15 with the saved number at center-ammo Y+20, using the radial name fade. It is independent of ammo availability and absent for unsupported weapons and Clank's command page. The engine cycles once per physical R1 edge only while the Triangle radial is open on an owned supported weapon. Both assignment and mode changes use the existing level SKILL_POINT jingle already used for cheat confirmation, without a cheat notification.

## Supported replacements

| Item ID | Weapon | Mode 1 | Mode 2 |
| --- | --- | --- | --- |
| 16 | Pyrocitor | Lava stream | — |
| 17 | Mine Glove | Flame mine | — |
| 13 | Visibomb Gun | Grenade launcher | — |
| 23 | R.Y.N.O. | Continuous missile fire | Shotgun |

Mode 0 is native. `apply_registry` changes only the Mine Glove, Visibomb and two mutually exclusive RYNO enable flags. Pyrocitor uses `effective_pyro` because its tuning lives in a typed structure. The raw INI stats remain unchanged. The same effective stats determine behavior, HUD capacities, ammo clamping and upgrade refills. A manual mode switch never creates ammo; switching to a smaller capacity uses the existing clamp.

Bomb Glove clusters, Pyrocitor triple flame, Suck Cannon crate suction, Drone Device companions, Tesla dual beams and all detailed settings remain INI/launcher-only. Adding a future mode requires deliberately extending the catalog, bounds validation and companion version; do not infer modes from generic experimental fields.

## Companion version 8

The v8 encoding appends eight bytes before the existing SHA-1 checksum:

- four item IDs: Up, Right, Down, Left; zero means unassigned;
- four mode bytes: Pyrocitor, Mine Glove, Visibomb, RYNO.

Bounds and duplicate nonzero bindings are validated before encoding and after decoding. The original content-hash/native-slot binding, atomic prepare/commit and checksum remain in place. Decoding v1–v7 supplies empty bindings and mode 0 without changing their existing progression/reward interpretation. Native saves themselves are untouched. Reading a companion for another native save is rejected as before.

## Porting and checks

The runtime hooks are small: hero and item pad normalization, paused grid callbacks, radial draw/cycle, and one effective Pyrocitor accessor at the existing engine configuration point. No assets or level classes are patched for these controls. When rebasing, retain upstream input, inventory and menu changes, then reapply these integration points; do not transplant entire old upstream files blindly.

Run rc-game library tests and `controls_api`, `golden_render_api`, `encounter_effects_api`, then build rc-engine. Launcher tests cover migration, capability 82, five-language help and group ordering. Native Windows installation additionally builds the GUI EXE, validates PE x64, runs its Tk/frozen smoke test, verifies runtime patch/capability and promotes both executables only after success. This repository does not contain original game data; final visual/controller testing requires the user's extracted data.
