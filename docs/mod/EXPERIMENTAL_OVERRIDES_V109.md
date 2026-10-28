# Alternate-only weapon tuning — v109

Local development patch **0.1.6.109**, runtime configuration capability **90**,
launcher **0.29.3**, for **ReRAC 0.3.6-alpha**. Public mod version stays 0.3.
No GitHub branch, release, tag or asset has been updated for this test.

## Per-grade fields

All three fields belong to each weapon's experimental settings frame, at grades
1 through 8. Standard values retain their existing place and meaning.

| Weapon | INI key | Units / limits | Initial value |
|---|---|---|---|
| Pyrocitor | `lava_damage_bonus_percent` | Additional damage percentage, 0..1000 | 25 |
| Mine Glove | `flame_mine_max_active_mines` | Integer deployed-mine limit, 1..32 | Copies that grade's `max_active_mines` |
| Mine Glove | `flame_mine_lifetime_seconds` | Seconds from release, 0.5..600 | Copies that grade's `mine_lifetime_seconds` |

Lavagun damage is `damage_per_second * (1 + lava_damage_bonus_percent / 100)`.
For example, 100 HP/s with 25 gives 125 HP/s. Zero adds no damage. The normal and
triple native flames keep the standard grade damage; a triple Lavagun applies
the bonus once to its shared stream damage clock, not once per overlapping lane.

The mine overrides are read only when the mine is launched in flame mode.
Normal-mode shots use the standard maximum and duration even if experimental
values are present. The existing shared deployed-mine budget is retained:
launching over the selected limit retires the oldest excess mines without damage.
This change does not create separate standard/fire mine pools.

## Mode selection and launch ownership

INI activation works when R1 Experimental is Off. When R1 Experimental is On,
the saved manual selection and grade's `r1_experimental_max` determine the active
mode. Mode 0 uses normal parameters; mode 1 selects Lavagun or flame mines.
The INI enabler alone cannot force alternate overrides while manual mode is 0.

`Evolution::effective_pyro()` already applies the manual Pyrocitor mode before
hand-item updates. The new bonus lives in `pyro_lava::Settings` and is validated
with the other lava options. `hero/pyrocitor.rs` multiplies only the lava branch's
`StreamSnapshot` at firing. Existing droplets retain that damage snapshot.
Normal-flame templates, intervals, ammo use, XP acceptance and reaction cooldowns
are not modified. XP still follows accepted HP loss through the existing pipeline.

For mines, `controls::apply_registry()` resolves the manual fire-mode flag before
`mine::experimental::snapshot_configured()` reads it. The snapshot chooses the
standard or experimental maximum and duration, then stores both in the released
mine's private extended pvars. `mine::release()` uses the captured maximum for
retiring excess mines; the existing update timer uses the captured duration.
Changing mode, grade or live profile does not rewrite already deployed mines.
Their independent damage, palette, size, lighting, sound and cleanup are retained.

No save format is changed. XP, grades, saved manual modes, randomizer seed and
locations remain compatible with existing saves.

## Migration and editor contract

The new keys are registered in `EXPERIMENTAL_DEFAULTS`, parser allowlists, numeric
ranges, labels/help and the five-language catalog. Their experimental membership
keeps standard controls first, enablers at the top of the alternate frame and
colors at its end. Each grade remains independent.

Migration adds only missing values. Existing explicit overrides are preserved.
Missing mine overrides inherit that same grade's personal standard values, not
another grade and not arbitrary new global defaults. The Lavagun bonus defaults
to 25, including old direct-run profiles that omit it. The optional embedded
profile and public example/default are kept in sync in this source snapshot.

Profiles with these fields require capability 90 even when the mode is Off;
older runtimes cannot parse the new keys safely. The launcher refuses a stale
runtime before launching or applying a live profile. The existing live-update
path can change the tuning for future shots without changing a saved seed.

## Local updater

The downloadable test kit contains exact-block runtime edits and the full new
launcher source. It accepts recognized v106 sources, preserving unrelated text,
checks the payload before changes, then runs the game library tests and native
Windows release build. The built runtime must report patch v109, capability 90
and editable ReRAC 0.3.6. The standalone GUI must pass its frozen page, scrolling
and new-field self-tests before installation.

The updater backs up changed source, runtime, launcher and profiles. New fields
are migrated in the selected `weapons.ini` and existing named presets, retaining
all old balance values. On failure, changed source and playable files are restored;
backup/restore records remain available. It does not reset saves or change the
selected runtime/data/save paths in launcher settings.

The normal update replaces the existing main launcher together with the runtime
and migrated profiles. The explicitly labeled Python-only fallback keeps the old
EXE but requires opening the new Python launcher: the old EXE does not understand
the new fields. The package BAT requires a successfully verified standalone EXE.

The new launcher retains complete page preloading and the v108 scroll changes.
Native Windows painting still requires user confirmation; this patch adds no new
window styles or page-layout mechanism.

## Verification and limits

- 1,329 rc-game library tests passed, including actual firing with normal versus
  lava modes at every grade, launch-time DPS snapshots, mine mode-only maximum
  and duration snapshots, old-profile fallbacks, R1 flag resolution and validation.
- Four public API tests passed for lava drawing/cleanup and saved controls.
- 181 launcher/profile/tool tests passed, including personal-value inheritance,
  BOM/CRLF/comment retention, presets/session export, ordering, localization,
  capability rejection and historical build/profile regressions.
- Six test-kit installer regressions passed, covering preflight and idempotence,
  unknown/corrupt inputs, compile/runtime/GUI/proof failure rollback, successful
  profile inheritance and rollback after a late profile-write failure.
- Real Tk 9.0 source self-test passed: all 17 pages, all eight grades, five
  languages, three layouts, new controls, palettes, scrolling and preserved widgets.

Rust checks use Linux, the locked dependency graph, no default features and the
BFD linker. This host did not build a linked native Windows engine or frozen GUI.
Those checks run on the authoring PC; in-game damage feel and Windows appearance
still need playtesting. No game data or generated executables are in this kit.

## Ownership and credits

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, is not a developer of ReRAC or the original games, and claims no rights to
the original games or their assets. Thanks to the ReRAC developers for the port
and the work that makes mods like this possible. Existing licenses and credits
remain applicable.
