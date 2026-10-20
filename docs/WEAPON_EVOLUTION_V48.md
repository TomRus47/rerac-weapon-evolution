# Weapon Evolution v48 implementation notes

Independent mod for ReRAC 0.3.4-alpha. Runtime patch identity is 0.1.6.48,
configuration capability is 48 and standalone launcher version is 0.10.0.
The incremental author installer starts from the attached v47 source revision.

## Pause-menu XP bar

`PageMenu::draw_with_evolution` receives the active game's immutable Evolution
profile. Existing `draw` callers keep the native behavior with no profile.
The engine supplies the live profile to its page-menu draws, including freeze
redraws. No extra global progress cache, save format or XP mutation is added.

The localized weapon-name label already identifies native normal/Gold names
before appending its grade. It now returns a port-only 0x20 metadata bit when
that exact name was drawn. Native panel-blit bits are unchanged. Only the Weapons
page (kind 3) collects the matching screen-space panel rectangle. Bar draws are
appended after panel composition/static, outside every PanelBegin/PanelEnd pair,
so the name panel cannot clip them. Descriptions, ammo labels, gadgets, Help and
Quick Select do not get a progress bar.

The outer span matches the full name-panel width, and begins immediately below
its bottom edge. Menu rectangle coordinates compensate for the native minus-one
pixel convention. Interior fill uses weapon_fraction for the selected item,
with the existing HUD colors and configured XP bar height. V5 is full with the
existing maximum-grade color. Disabled/non-evolving items, including the wrench,
produce no bar. Changing selection or loading another save reads that profile's
current values directly.

## Ten-drone capacity

`drone::MAX_DRONES` is shared by gameplay and the menu preview. Slots expand from
six to ten; target reservations expand to MAX_DRONES+1, retaining room for the
round's newly assigned target. Creation reserves an empty slot before creating
a moby, and a failed/full launch does not report success or consume ammo.
Per-grade drone_count accepts integers 0..10. Zero suppresses native drone
creation; companion-only/combined/Sonic Summoner experiments retain their mode
selection. Existing profiles keep every saved count, with no automatic balance
increase. Set drone_count=10 in a desired Drone Device grade to use all ten.

Formations above six use stable slot order rather than moby-update order, one
phase sampled per game tick and a common height/plane. Their radius preserves
at least the normal six-drone centre spacing, enlarged for configured size and
observed model bounding spheres. Shared angular placement avoids bunching from
independent angle/tilt springs. Refill/removal reforms the larger ring as a whole;
ordinary rotation then advances continuously. Native terrain collision remains,
and an off-screen stuck recovery retains each drone's own orbit offset rather
than sending all of them to one point. Terrain can still constrain the ring in
very confined spaces; in-game visual checks near walls remain necessary.

The size spring now targets class_scale * drone_size_multiplier instead of
undoing the per-grade scale after creation. The menu preview also uses that
scale, allocates ten Drone roles and avoids its old modulo-six positions for
larger populations. The companion singleton and weapon damage/XP are unchanged.

## Configuration and installer

No new INI keys are introduced. drone_count remains a standard field before the
experimental footer. The launcher accepts ten, documents the range and requires
capability 48 for profiles containing it; the installer verifies exact v48 runtime
identity and launcher 0.10.0. Source and launcher/INI backups, preflight integrity,
reapply/restore, custom-icon preservation and existing embedded-profile protection
continue to apply. No audio probes, executables or disc data are included.

## Automated and visual validation

Regression coverage checks bar width/position, independent selected progress,
maximum-grade/disabled items, all five drone grades and malformed capacities.
Geometry tests check every pair for populations 7..10 and several model sizes;
the moby-world test creates all ten, rejects an eleventh, orbits them, removes one
and refills its slot while checking separation after each tick. The generic test
hero receives real identity rotation rows, as the in-game matrix builder supplies.

See the build kit's VALIDATION.md for results and remaining Windows gameplay
checks. A Windows target type-check does not replace an in-game visual test.
