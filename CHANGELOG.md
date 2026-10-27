# Unreleased — v106 follow-up

Gameplay **0.1.6.106**, capability **89**, launcher **0.28.9**, ReRAC **0.3.6-alpha**. Source/main update only; public version remains **0.3**. Existing release/tag/downloads are unchanged.

- Match the experimental wrench slam's maximum visible radius and centre to the actual damage sphere. Replace unbounded decorative bomb particles with a filled, independently colored translucent burst; preserve HP damage, XP, impact sound and once-per-attack behavior.
- Include the confirmed local slam fixes: forward surface probing, five-tick lead, native bomb object messages, accepted per-instance wrench damage and safe moby-grid queries at ground/world boundaries.
- Include normal-frame red damage feedback, finite non-stacking burns and immediate fire on accepted hits before one-hit victims disappear.
- Include complete HUD batch/index preservation and localized, independent bar position/stacking controls. Keep saved values and additive migration stable.
- Show one bright sky-blue explosion when earned Nanotech XP increases maximum health; retain the established weapon-grade upgrade colors and sizes.
- Add collision/HP/XP/visual-radius regressions, reproducible transactional Windows delta tools, profile migration coverage and English implementation/build documentation.

# Unreleased — Weapon Evolution 0.3 development snapshot

Gameplay 0.1.6.83; configuration capability 84; launcher 0.27.1; ReRAC 0.3.6-alpha. Main/source update only; previous release tag and assets unchanged.

- Synchronize the complete supplied 0.3.6-alpha workspace and the accumulated local mod changes: encounter tuning/population, Nanotech XP/health, seeded weapon discoveries, lava streams, NG+ Gold progression and eight weapon grades.
- Add always-available R2/Circle firing and optional saved D-pad bindings for weapons and all equippable gadgets, with colored borders and a Weapons-only assignment caption.
- Add contextual saved radial R1 modes for Pyrocitor, Mine Glove, Visibomb and RYNO; leave optional enhancements and Suck Cannon crate suction in the launcher.
- Correct Gadgets page scope and use normal menu confirmation for assignments/mode changes.
- Repeating a boots/helmet shortcut removes the equipped item; repeating a Clank-pack shortcut cycles owned Heli/Thruster/Hydro packs after its first selection. Respect pending inventory requests, ownership and native gameplay restrictions.
- Support both L1 and R1 crouching in direct gameplay while retaining contextual radial R1 mode selection.
- Update English implementation/build/save documentation, five-language launcher help, configuration reference, source provenance/index and legacy regression fixtures for eight grades. Keep default balances, presets, custom icon and save compatibility.
- Separate development identity from the existing published descriptor and require explicit manual draft authorization in release automation.

Historical published/local notes follow.

# Weapon Evolution 0.3 — updated build

Gameplay 0.1.6.59; configuration capability 58; launcher 0.17.2.

- Align experimental ammo consumption and inventory with the displayed capacity.
- Clear flame-mine lights on respawn and fix enlarged-mine landing/pending throws.
- Restore normal launcher page rendering and scroll dropdowns without changing selections.
- Fix guided Visibomb control with increased duration; add independent experimental grenade/mine model palettes.
- Apply flame-mine size to the native mine, alternate bomb and alternate sphere using each selected model’s reference scale.
- Include a matching Windows runtime, standalone launcher, source archive and current documentation.
- Clarify independent mod authorship and original-game rights throughout the published material.

Historical local patch notes follow.

# Local v58.2 (unpublished, launcher 0.17.2)

- Allow launcher restoration directly in source/launcher folders without a game runtime or weapons.ini.
- Open the restored editor without probing a game; actual game launch keeps its normal validation.
- Accept a source root or bundle root when it contains the launcher subfolder.
- Preserve the v57 page restoration, all v58 gameplay changes and existing profiles.

# Local v58.1 (unpublished, launcher 0.17.1)

- Restore the original v57 page, navigation and scrolling lifecycle after incomplete rendering on Windows.
- Remove page caching, offscreen page construction, background warmup and composited painting.
- Preserve all v58 model selectors, independent palettes and gameplay fixes.
- Add a launcher-only restore BAT with backups; no game recompilation is required.

# Local v58 (unpublished)

- Keep complete launcher pages cached; prepare layout changes offscreen and buffer eligible Windows painting.
- Fix native Visibomb steering with increased flight duration and honor captured turn strength.
- Add per-grade grenade and flame-mine model selectors with isolated palettes and localized controls.
- Align the experimental missile model with its rising/falling velocity instead of spinning.
- Preserve v57 settings, saves, presets, icon and all earlier fixes; require configuration capability 58.

# Local v57 (unpublished)

- Align experimental ammo limits with real inventory and consumption.
- Clear flame-mine point lights on death reload; resolve enlarged hulls out of floors.
- Retry pending mine allocation and charge only successful releases.
- Scroll the launcher page over dropdowns without cycling settings.

## Weapon Evolution 0.1.6.54 (local test only, launcher 0.14.0)

- Evolve the OmniWrench through five saved grades, with actual hit HP, proportional slam damage, throw range/speed and additive passive bolt attraction.
- Move its Gold appearance to each grade; add top-left-only HUD and moving-combo switches plus a crate-only ground-slam Crate Breaker.
- Preserve v1/v2 progress when reading new WEVO-v3 companion saves; native save format is unchanged.
- Prioritize brighter R.Y.N.O. pellet cores over optional trails, reclaim own trails under pressure and retry missing cores without changing physical damage.
- Fix Visibomb grenade markers and air detonation at the range boundary; continue falling to actual contact, explode on water and remove silently in death regions.
- Add Devastator firing sound to successful grenade launches. Preserve all other v53 settings and the supplied icon. No GitHub update or release.

## Weapon Evolution 0.1.6.53 (local test only, launcher 0.13.0)

- Bright shared grade icons; V4 orange and V5 golden yellow across all evolving weapons.
- Standard per-grade Visibomb missile blast radius, captured at launch.
- Optional Visibomb grenade launcher: free player movement/camera, native bomb spin, landing marker and throw feedback; editable size/arc/range/interval/HP/colors; explosion-owned Visibomb XP.
- Experimental flame mine size (150% default), matching body/flame rotation and scaled ground clearance.
- Preserve existing profile values, icon and saves. Local source/build/open kit only; no GitHub update.

## Weapon Evolution 0.1.6.52 (local test only, launcher 0.12.0)

- Configure four collected-bolt unlock milestones: Taunter 10,000; Suck Cannon 25,000; Walloper 50,000; Morph-o-Ray 75,000.
- Expose eleven normalized ammo pack probabilities, including zero to exclude a type and an actual-chance preview.
- Start new games with no Bomb Glove ammo while ammo-unlock mode is active; native planet-entry grants cannot refill/equip the locked glove. Preserve existing-save ammo.
- Group Global Settings into Base settings and Experimental ammo-unlock mode, with the enabler first in its card.
- Add a complete local source-overlay/build/open kit with personal-profile migration, backups, rollback on write failure and reuse of Cargo's target cache. No GitHub update or launcher EXE rebuild is required.

## Weapon Evolution 0.1.6.50 (public 0.3.2, launcher 0.10.0)

## Weapon Evolution 0.3.3 / gameplay 0.1.6.51

- Optional ammo-based weapon unlocking, independent crate packs, Gold-Bolt pack probabilities and collectible-bolt permanent unlocks.
- Per-slot mode state with old XP companion migration; weapon/ammo shops disabled only in this mode.
- R.Y.N.O. ammo in ordinary randomized crates, all-weapon Gold appearance and silent secondary Tesla acquisition.
- English launcher 0.11.0 with lossless profile migration. See [release notes](docs/releases/v0.3.3.md).

- Admit all 35 R.Y.N.O. shotgun pellets before charging ammunition, starting cooldown or queuing effects; retry without charge when the dynamic object table is full.
- Prioritize shotgun heads over optional trails, preserve their first-frame anchors, and cap optional tail allocations under shared particle pressure.
- Enforce pellet lifetime as well as range/collision expiry, including floating-point range residue, and install the experimental update independently of native per-level update entries.
- Reuse immutable overlay instruction/address indexes during level recognition, prepare menu/map tables on the loader worker and share section storage with consumers.
- Release temporary recognition indexes after preparation; preserve final per-level tables, balance, configuration, saves and the launcher icon. No audio probes are added.

## Weapon Evolution 0.1.6.49 (public 0.3.1, launcher 0.10.0)

- Reset deferred draw callbacks before installing a compacted death-reload table; reproduce the Oltanis arc panic in a regression fixture.
- Clear table-bound companion/reaction/arc references, release arc-owned lights, and preserve XP, checkpoints, profile values and loaded geometry.
- Refresh English developer/configuration/release documentation, source integrity checks and Windows draft-release tooling.
- Publish the latest supplied v48 profile as the tracked default without replacing installed personal profiles.

## Weapon Evolution 0.1.6.48 (launcher 0.10.0)

- Show the selected weapon's XP bar immediately below its name/grade panel on the pause Weapons page, matching that panel's full width. Use the live profile and keep V5 full.
- Expand Drone Device gameplay slots, target reservations and menu-preview slots to ten. Keep larger formations in stable slot order on a shared phase/plane with model-aware clearance.
- Preserve the configured drone size during orbit, support per-grade drone_count 0..10 in the runtime and launcher, and require configuration capability 48 for drone profiles.
- Preserve existing per-grade counts, balance, presets, saves and the custom launcher icon.

## Weapon Evolution 0.1.6.47 (launcher 0.9.0)

- Exclude the camera's own invisible class-1007 helper from every locked follow-camera query, preventing self-collision during downward aiming while retaining scenery and other object collision.
- Release the experimental R.Y.N.O. firing stance on trigger release, blocked use or empty ammo, including during cooldown. Preserve the firing clock to prevent tap-to-fire rate bypasses.
- Keep the v46 35-pellet grid, independent spread axes, configuration values and launcher icon unchanged.

## Weapon Evolution 0.1.6.46 (launcher 0.9.0)

- Keep locked aim independent of collision-resolved position and arm length, preventing steep-angle terrain jitter and unwanted sideways turns.
- Give experimental flame mines two-sided centre/lower-hull terrain sweeps and volume contact; enable their native collision volume after landing.
- Expand the R.Y.N.O. shotgun to a 7-by-5 muzzle grid with independent per-grade horizontal/vertical spread. Zero spread keeps 35 distinct parallel origins; centre spread is zero.
- Migrate the previous spread value to the horizontal axis and add a restrained 3-degree vertical default, preserving explicit per-grade edits.
- Cancel active and queued upgrade notices on death, clock rewind or restored lower weapon levels, without changing saved XP.
- Keep generated embedded INIs and prior creature tests outside source replacement, and preserve presets, balance and the launcher icon.

## Weapon Evolution 0.1.6.45.1 (launcher 0.8.0)

- Expand locked vertical view and aim to 85 degrees above/below the horizon, with independent requested pitch so camera collision cannot narrow the aiming cone.
- Preserve the ten-tick input ramps, selected camera speed and native unlocked view.
- Correct the v45 author installer to preserve earlier creature tests and validate the actual launcher folder before source writes.

## Weapon Evolution 0.1.6.45 (launcher 0.8.0)

- Add an optional Drone Device Sand Mouse companion, shared singleton with native dens, independent lifetime/shot settings and Drone Device XP.
- Restore two independent native Tesla electrical chains, including free firing; configure the secondary ray's same-target damage (25% default).
- Fix R.Y.N.O. real-HP terminal damage, including Blackwater City tanks, and detonate at maximum travel/lifetime.
- Add continuous-missile and nine-barrel shotgun R.Y.N.O. experiments, independent flame mines, deployment limits/lifetime/colors, and Taunter stun launch distance.
- Preserve INI/preset balance and experimental footer ordering; embed and distribute the supplied launcher icon.

# Changelog

## v0.3.4-alpha

Fixes from playing v0.3.3.

- Cutscenes: characters are lit like the place they stand in, Ratchet's head is the right size, and Clank's eyes and antenna glow.
- Lamps, glowing mushrooms and other lights brighten Ratchet again, like in Novalis's caves, Aridia's factory and Al's shop.
- Ratchet raises his gun when standing close to a wall, and his weapons fire along it.
- Aiming with the wrench or the Swingshot shows the reticle.
- The Swingshot's claws fold out once and stay open.
- The Comet-Strike sparks when the wrench hits a wall or an enemy.
- The Bomb Glove's blast throws enemies away from it, and the gold Bomb Glove has its bigger, golden blast.
- Pressing Start on the Hoverboard asks to quit the race.
- Shop messages fit inside their box.
- Novalis: Drek's gunship destroys the bridge only once.
- Aridia: the lifts come for Ratchet from either end, and the shark and nest counters show during the race.
- Kerwan: Ratchet no longer falls off the taxis mid-ride.
- Blarg Station: the pad Clank uses carries Ratchet, the airlock doors stay in step after Clank's section, and the queen's screams and death play out as they should.

## v0.3.3-alpha

Fixes from playing v0.3.2.

- Gemlik Base: Qwark's shield drops again, he flies off when the fight starts, the camera follows his crash, and his health bar goes away afterwards.
- Gemlik Base: after the fight the new ship is waiting, closed, with its orange canopy, and Ratchet can board it.
- Veldin Orbit: the nebulae out past the fleet are back, the windows have their glass, and the two pools near the start have water.
- Quartu: the patrolling guard walks its route again, so the Hologuise gets Ratchet past the barrier.
- Hoven: Ratchet freezes in the icy water.
- Veldin: after dying, the pools drained with the Hydrodisplacer stay drained.
- Blarg Station: broken objects no longer leave oversized pieces behind.
- First person: only a thrown wrench shows in Ratchet's hand.
- The Drone Device works from the pause menu, and its drones show up next to Ratchet there.
- The map page on Veldin shows the right name.
- Cutscene dialogue plays in the chosen language.

## v0.3.2-alpha

Fixes from playing v0.3.1.

- Windows: the black window on some PCs is fixed.
- Gold weapons look gold, in Ratchet's hand, in the menus and in the cutscene that now plays after buying one.
- Ratchet gets darker indoors, like in Veldin's garage, and so do his weapon and Clank.
- Ratchet glares at enemies, and makes faces when hurt, holding his breath or pulling out a big weapon.
- Things that should stand on the ground no longer float: Qwark at Oltanis's Gadgetron stand, Umbris's Hydrodisplacer spots and more.
- Oltanis: the round tilting platforms are solid again.
- Kalebo III: the lifts go all the way up and down.
- Gemlik Base: Ratchet no longer dies outside the base during the space battle, and the asteroids drift.
- Umbris: the turrets fire from their guns.
- The map: Ratchet's arrow turns the right way, the markers are in the right places, and the map no longer goes blank after skipping a cutscene.
- The oxygen meter no longer shows up after drowning.
- Options: the pages keep their layout after leaving Port Options, which now scrolls.

## v0.3.1-alpha

Fixes from playing v0.3.0, smoother frame pacing, and new graphics options.

- Dying no longer empties every weapon.
- Gemlik Base: Ratchet wears his O2 Mask from the start.
- Clank and Giant Clank no longer get stuck in first person.
- The weapon counter no longer keeps the vendor's ammo icon, and each weapon shows its own ammo.
- Vendor: one weapon per press when browsing, and the selection frame shows again.
- The Comet-Strike works on Magneboots walls and ceilings.
- The Metal Detector's head points at what it scans.
- The boots hide Ratchet's feet.
- Gadgets and Weapons pages: Ratchet plays each item's animation on every planet, and the Heli-Pack spins and folds.
- Jet fighter missions: locking on to a target close to the ship no longer crashes the game.
- One game step per frame, at up to 60 frames a second: no more doubled or skipped frames, and less work for the computer.
- New graphics options under Options, Port Options: a preset, anti-aliasing, detail distance, texture filtering and HUD scaling. Every option starts at Original, the PS2's picture on a TV, so the game now looks soft like the original. Set Anti-aliasing to Off for the sharper look of earlier versions.
- The Port Options page fits all its rows.
- A Linux version (64-bit PCs), not tested yet: reports welcome.

## v0.3.0-alpha

Dying now works the way it does in the original, and a long list of fixes from play-testing Umbris, Orxon, Gemlik and the rest.

- Dying brings back enemies, crates and pickups the way the original does, and saves remember which enemies stay defeated and what they dropped.
- Lifts and gates you already opened stay open after loading a save.
- After dying, the ship can be boarded and has its glass, nanotech no longer gets stuck in the air or turns see-through, and leftover fog no longer lingers.
- Kerwan: the troopers and the info bot are on the train.
- Umbris, Gaspar, Orxon and the last two planets: breaking ammo crates no longer crashes the game, and the vendor sells ammo again.
- Umbris: the first landing no longer leaves the screen dark with Ratchet missing.
- The Snagglebeast: its fire sweep, suck beam, shockwave and shield work like the original, and rapid fire no longer lands every shot. A few other enemies had the same problem and are fixed too.
- Boss health bars no longer freeze after dying.
- Orxon: the gate at the start of Clank's section opens without dying first, dying there brings Clank back at the checkpoint, and Clank no longer gets stuck after the landing or the turret cutscene. The gadgetbots' glass bubbles show.
- Gemlik Base: Ratchet walks the curved Magneboots rails, and the robots' rolling fire wall shows and hurts.
- The Suck Cannon pulls in Eudora's robots, the crew on Drek's fleet and the hoppers on Veldin.
- Ratchet no longer holds the wrench on ledges or while swimming.
- Space travel: planets show their real colours, and the ship's canopy stays in place in every flight and cutscene.
- The planet preview in the ship's menu turns smoothly, and the "Enter Ship" prompt no longer shows behind the planet menu.
- Qwark's broadcast on Rilgar no longer takes on the TV screen's tint.
- Vendor: weapons face the right way, and the item bar fits the list.
- Ammo and other pickups no longer show their insides as they fade in.
- Ratchet, the crate and the ship stay on the title screen behind the main menu.

## v0.2.3-alpha

Mostly Blarg Station, plus fixes for Clank's sections and a few other planets.

- Blarg Station: the energy doors' beams cross the right way, Clank's bridge extends once he has the Hydrodisplacer, and freed gadgetbots no longer fly off and die. Their command voices play.
- Blarg Station: the big red button's cover opens once the room is clear, and the self-destruct countdown shows on screen.
- Blarg Station: the countdown voice lines play, and the escape, the shuttle flight and the launch tube have their explosion sounds.
- The queen's health bar on Blarg and the Umbris boss's health bar go away when the boss dies.
- Dying in a Clank section no longer brings you back as Ratchet, leaves Clank unhurtable or leaves a frozen Ratchet standing around.
- Clank: the pause menu greys out what he can't use, ledge climbs work, and his antenna glow and heli-pack stay in place when he turns.
- Ratchet's ship is solid again: you can no longer walk through it.
- The info bot movie from Drek's henchman on Eudora plays once.
- The quick select no longer opens during scripted scenes or when △ is used on a prompt.
- Info bot and other messages at the top of the screen no longer stay up too long.
- The weapon counter keeps its shape in widescreen.
- Oltanis's lightning thunders.

## v0.2.2-alpha

- Fixes a black screen after the opening movies of a new game, after loading a save and after flying to another planet.

## v0.2.1-alpha

Fixes from the first round of testing, and new display settings.

- Port Options has three new settings: Aspect ratio (4:3 as on a TV, or 16:10 and 16:9 widescreen), Resolution (Window, or 416p up to 2160p) and Fullscreen. Widescreen shows more of the world at the sides while the HUD and menus keep their shape. F11 also switches fullscreen.
- Fullscreen and maximised windows now have black bars, and the menus no longer break in them.
- Loading a save on Kerwan brings back the enemies of later missions, and Helga no longer offers the Swingshot again once you have it.
- Reaching the info bot on Kerwan's train no longer throws Ratchet off the map.
- The Qwark bot on Kerwan no longer repeats its cutscene.
- A thrown wrench brings back the bolts and ammo it hits, as in the original.
- The bomb glove no longer floats in the air during cutscenes, at the vendor or when boarding the ship, and Clank's antenna glow no longer floats in cutscenes.
- The ship's canopy shows during take-off and landing.
- The planet selection screen shows the button hints and your gold bolt count.
- Enemies and objects that could turn invisible while still active now show.

## v0.2.0-alpha

The first test build of ReRAC. It's an early alpha: every planet is in, from Veldin to Drek's fleet, but only the first few have been played through properly, so expect rough edges, missing details and the odd crash further on.

- Made for Macs with Apple Silicon. There's also a Windows build, but nobody has tried it yet, so it may not even start; please tell us how it goes. Linux comes later.
- You need your own copy of the game (the NTSC-U disc, SCUS-97199). No game files are included; the ReRAC launcher sets the game up from your disc.
- Veldin, Novalis, Aridia and Kerwan are complete. The other planets have all their enemies, machines and bosses, and are waiting for a full play-through.
- The main menu, saving and loading, flying between planets, the Gadgetron vendor, and cutscenes with their sound and music all work.
- If something goes wrong, please report it with the log the launcher keeps.

## Unreleased development: v102 / launcher 0.28.7

- Refresh enemy red damage feedback and finite body-fire cycles independently of stun/reaction clocks; include native creature classes without XP adapters.
- Add per-grade wrench visual ignition, Triangle-only name/XP visibility and an optional bomb-style ground-slam explosion with damage/radius/palette controls and wrench XP attribution.
- Hide custom weapon name/XP while swimming; add an alternative bar position between name and ammunition, and indigo/yellow/white weapon-name grades.
- Use the native ammo-crate mesh for model-free randomized rewards while retaining real Gold Bolts, Infobots and modeled objects.
- Verify exact extra-enemy counts in NG+ with difficulty scaling and unscaled native HP categories.
- Synchronize the supplied v97–v101 runtime/launcher fixes, add a checked local update kit and standalone ZIP helper, and retain current balance and release assets pending author playtests.
