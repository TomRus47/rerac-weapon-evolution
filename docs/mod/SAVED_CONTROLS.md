# Saved shortcuts, equipment repeat actions and weapon modes

ReRAC 0.3.8-alpha. This document describes the current source controls; local test kits can carry later runtime patch identifiers. Existing bindings and mode choices remain compatible.

## User behavior

Enable **D-pad Quick Select** at the bottom of the first Global Settings card. Ratchet moves with the analog stick. In paused Weapons or Gadgets, hold L1 plus one physical D-pad direction to bind the highlighted owned item. The cursor stays fixed even with the stick held. Up has a green outline, Right red, Down blue and Left pink. Assigning an item to another direction clears its previous binding. The assignment caption appears in Weapons only; Gadgets supports all fourteen native equippable entries without extra text.

During direct gameplay, a bound direction requests the corresponding native equipment slot. Repeating a boots or helmet binding removes that same equipped item; another press equips it again. Hand items retain normal selection behavior. For Heli-Pack, Thruster-Pack or Hydro-Pack, the first press selects the bound pack, then repeating that binding cycles owned packs in that order and wraps. Unowned packs are skipped. A different shortcut or an external backpack change restarts selection at the assigned pack.

These requests do not grant items or bypass native water, magnetic-surface, head/back, grinding, vehicle, turret or scripted-state restrictions. Wearing Grindboots does not itself block weapon selection. Confirmation uses class 0x472 sound 0, the ordinary weapon-menu confirmation, not the cheat jingle.

**L1 uses first-person aiming and R1 crouches** in direct Ratchet gameplay. Circle/R2 form one firing trigger; holding one while releasing the other does not stop fire. R2 aliases Square for the wrench. With Triangle's radial menu open, R1 is consumed by its contextual mode selector while gameplay is locked; it does not crouch there.

**Equipped gadgets also accept R2 wherever their native action accepts Circle.**
This includes Swingshot, Hydrodisplacer, Trespasser, Metal Detector, Hologuise and
PDA. Keep R2 held to maintain a held action; switching between R2 and Circle
without releasing both does not detach the Swingshot or trigger a second use.
While disguised, R2 performs the Hologuise salute, even with the wrench saved in
the hidden hand slot. Passive packs, boots and helmets keep their native controls.
Menus, puzzle controls, vehicles, turrets, cutscenes and alternate Clank bodies
continue to use their physical inputs. No additional INI setting is required.

With **R1_Experimental** On, selecting Pyrocitor, Mine Glove, Visibomb or RYNO in the Triangle radial exposes the native R1 glyph and `= 0/1/2` beneath ammo. R1 cycles once per press. Modes are normal/lava, normal/fire mine, guided/grenade and native/continuous/shotgun, respectively. Bomb clusters, triple flame, crate suction, Drone Device companions and dual Tesla remain launcher-only. Turning this master Off restores per-grade INI mode switches while retaining saved player choices.

## Implementation boundaries

- `weapon_evo_controls.rs`: catalog, validation, context gates, physical edges, normalized gameplay input and native inventory requests.
- `pad.rs`: unmirrored physical button snapshots captured before analog merge and gameplay locks. Analog motion cannot activate a binding, and radial lock 2 cannot hide R1 edges.
- `menus/pause/gadgets.rs`: real grid callbacks and borders accept Weapons kind 3 and Gadgets kind 4; Help/other grids are excluded. `pause/shortcuts.rs` owns the Weapons-only caption.
- `menus/quick_select.rs` and engine `menu_render.rs`: mode rendering, physical-edge cycling and normal confirmation audio.
- `weapon_evo.rs`: effective mode flags and ammo capacities; raw per-grade INI tuning remains unchanged.
- `weapon_evo_save.rs`: the historical v8 format appends four direction item IDs and four mode bytes to the existing content-hash/slot-bound companion, before its checksum. v1–v7 initialize empty bindings/native modes and preserve older state.

The shared `controls::hero_gameplay_pad` supplies the same semantic input view to
hero transitions, `items::items_update` and the deferred `gadgets::after_items`
context in `tick.rs`. Normalizing only the first two is insufficient: Swingshot's
deferred hook update reads the held fire mask and otherwise sees a false release.
Swingshot and Hydrodisplacer use phases have narrowly scoped input exceptions,
without granting `direct_control` or allowing equipment changes during those
phases. The Hologuise body maps R2 to Circle rather than the hidden wrench's Square.
Physical pad snapshots are never overwritten by this view.

Native `global.owned` validates gadgets, including NG+ saves whose acquisition flags were reset. Native `global.equipped` stores actual equipment. Pending session requests take precedence when deciding a toggle or cycle, including `inventory::NOTHING`; this prevents rapid presses during equip/unequip animations from acting on stale equipment.

`SessionState.shortcut_back_cycle` is transient bookkeeping: direction, bound item and last requested pack. A matching sequence advances through owned IDs 2/3/4; other equipment changes restart it. Session initialization clears it on load/travel/respawn. It is not a new persistent setting and does not alter the native save or companion format. The D-pad assignment never changes while backpacks cycle.

No runtime file I/O is added to input polling. Controls settings are published with the existing profile registry; manual mode updates republish effective values without resetting combat epochs or progression.

## Regression and gameplay checks

Tests exercise all fourteen gadget entries through actual page callbacks, both equipment-page kinds, normal confirmation, analog cursor freeze, four-slot routing, ownership/context rejection, pending and committed boots/helmet toggles, backpack wrap/skip/one-owned behavior, external changes and saved-binding stability. Input tests cover Circle/R2 unions and distinct L1/R1 actions, pressure, physical directions, mirrors and radial locks. Save tests cover old-version migration, checksums and cross-slot isolation. Additional gadget tests exhaust both-button press/hold/release transitions, cover pressure and buffered edges, verify gadget phase and menu/vehicle/body exclusions, and exercise the native deferred Swingshot pull consumer with R2, Circle, both and neither.

On Windows, compile the engine and test visual borders/caption, native equipment animations, native first-person/crouch shoulders, repeated shortcuts, R1 radial modes, audio, death, travel and independent save slots. Passing host-side tests does not replace these gameplay checks.

Weapon Evolution is an unofficial independent mod by TomRus47; the mod author does not develop ReRAC or the original games and owns no rights to their games, characters, trademarks or assets. Upstream licenses and contributor credits remain applicable.

Current WEVO v9 appends the mixed-randomizer state after the v8 fields. It decodes v1–v8 without discarding weapon, Nanotech or equipment progression. Preserve the native `saveN.bin` together with its entire matching content-addressed companion folder when backing up or reporting a run; never pair companions from a different native save.
