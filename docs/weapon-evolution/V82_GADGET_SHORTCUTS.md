# Gadget shortcut page scope and confirmation audio

The native pause menu switches its `PageMenu.kind` to the current page's kind after navigation. Weapons uses kind 3; Gadgets uses kind 4. The v81 shortcut callback and border renderer accidentally accepted only kind 3. Inventory requests already supported all four native equipment slots, but gadget assignment and its visual feedback could not be reached on the actual Gadgets page.

`gadgets::shortcut_page` now explicitly accepts both kinds and is shared by assignment and border drawing. It does not enable shortcuts on Help or other grids. `grid_update_with_shortcuts` accepts an explicit policy flag so regression tests can exercise the real focused-grid callback without modifying process-wide configuration. Holding L1 still freezes digital and analog cursor movement, validates ownership and assigns a single physical direction. Save format and normal equipment restrictions remain unchanged.

The assignment caption remains Weapons-only in `pause::shortcuts`; Gadgets has the shortcut functionality and colored borders without additional text. Head, feet, back and hand slots all use the same native session requests as before, with existing context restrictions.

Both pause assignments and radial R1 mode changes now request `MenuSound::Confirm` (class 0x472 sound 0), exactly as native weapon selection does. The special `ControlConfirm` enum value and its engine jingle branch have been removed. Existing cheat functionality and unrelated sounds are unchanged.

Regression tests cover all fourteen native gadget-grid entries: Swingshot, Hydrodisplacer, Trespasser, Metal Detector, Hologuise, PDA, Heli-Pack, Thruster-Pack, Hydro-Pack, Sonic Summoner, O2 Mask, Pilot's Helmet, Magneboots and Grindboots. Every entry is tested through the real callback on both page kinds, checking assignment, normal confirmation and cursor stability with the stick held. A gameplay test verifies hand/back/head/feet request routing and rejection after ownership is removed.

Internal runtime patch: 0.1.6.82, configuration capability 83, ReRAC 0.3.6-alpha. Launcher 0.27.0, INI, presets, save companion v8 and stored assignments/modes require no migration or rebuild. This is a local follow-up to v81 and performs no publication.
