# Local v55: launcher organization and alternate ammo capacities

This is an unpublished test update for Weapon Evolution on ReRAC 0.3.4-alpha.
No GitHub repository, tag, release draft or asset is changed by the local kit.

## Player behavior

- All binary controls, including OmniWrench controls, use **Off / On** dropdowns.
  Off writes `false`; On writes `true`. Existing `yes/no/1/0` profile values are
  displayed correctly and remain untouched until the player changes them.
- Each grade has a Standard settings frame and, where applicable, an Experimental
  settings frame. Experiments begin with their mode switches, then their alternate
  ammo capacity. Ammo precedes damage, area controls, reach, and other settings.
  Gold controls are the last non-color standard fields. Colors finish their own
  frame. R.Y.N.O.'s two mutually exclusive switches remain together at the top.
- Mine flame core/outer/burn colors belong to the experimental frame. The normal
  mine explosion and light colors remain standard because native blasts use them.
  Companion/shotgun/grenade colors remain experimental. Existing keys are retained.
- Navigation and INI sections use: Bomb Glove, Pyrocitor, Blaster, Glove of Doom,
  Suck Cannon, Taunter, Mine Glove, Devastator, Visibomb Gun, Walloper, Decoy Glove,
  Drone Device, Tesla Claw, Morph-o-Ray, R.Y.N.O.; OmniWrench remains last.
- The game mode is named **Ammo-Survival Mode** in every launcher language. Its
  internal key remains `experimental_ammo_unlock_mode` for preset/save compatibility.
- Language can be changed between English, Italian, French, German and Spanish.
  Names, field labels, help, menus, dialogs and ordinary status text update. The
  setting is stored beside other launcher options, independently of presets and
  the game's language. Switching languages preserves unsaved values, the current
  preset and any running game session. Technical INI addresses, numbers, RGB codes,
  filenames, paths and original diagnostic details remain unchanged.

## Experimental ammo contract

`experimental_max_ammo` is an integer in each of the five grades of:

| Weapon | Mode switch | Capacity limit |
|---|---|---|
| Bomb Glove | `cluster_bombs_enabled` | 1–999 |
| Pyrocitor | `triple_flame_enabled` | 1–999 |
| Suck Cannon | `crate_suction_enabled` | 0–10 stored projectiles (native minimum 1) |
| Mine Glove | `flame_mine_enabled` | 0–9999 |
| Visibomb Gun | `grenade_launcher_enabled` | 0–9999 |
| Drone Device | `sand_mouse_enabled` | 0–9999 |
| Tesla Claw | `dual_beam_enabled` | 0–9999 |
| R.Y.N.O. | `continuous_missiles_enabled` or `shotgun_enabled` | 0–9999 |

When a listed mode is enabled at the current grade, its alternate capacity wins.
Otherwise standard `max_ammo` wins. Bounds match the existing standard capacity; a zero Suck Cannon capacity retains
the native one-slot safety clamp. Suck Cannon substitutes its `capacity` instead:
it remains a suction-storage weapon and does not gain ordinary ammo pickups.
No-ammo weapons without an ammo-consuming experiment receive no artificial counter.
An enabled companion switch uses the alternate capacity in all its submodes.

Migration initializes missing alternate capacities from the same grade's existing
standard capacity, preserving custom balance. Explicit alternate values always win.
The Rust parser also accepts older profiles without the key; those retain standard
capacity even if an experiment is enabled. Loading a profile grants no ammunition.
The common capacity publication supplies HUD, pickups and shops; Suck Cannon's HUD
and held-slot checks read the same effective `capacity`. Grade changes republish it.
Capacity selection does not modify damage, XP, reaction clocks or projectile data.

## Implementation map and maintenance

- `launcher/schema.py`: canonical section order, `field_order`, `field_groups`,
  experimental enabler list, accepted fields and bounds. Register future mode
  switches explicitly in `EXPERIMENTAL_ENABLERS` and put their capacities in the
  experiment catalog. Do not infer mode enablement from crate filters or colors.
- `launcher/migrate_ini.py`: add missing keys, preserve values/comments and BOM/CRLF,
  normalize sections and frames in an idempotent pass. Layout may change as requested.
- `launcher/i18n.py`: built-in catalogs; no network or translation package at runtime.
  English field definitions remain in `schema.py`. Field/help rows use stable keys;
  UI message rows use English source text. Add every language when adding a field.
  Technical addresses remain available in help for manual INI editing.
- `launcher/launcher.py`: display variables map dropdown labels to canonical values.
  Language changes rebuild labels only; they do not reload the document or save a
  preset. Context-menu restore/copy rebuild the displayed controls too.
- `launcher/build_launcher_exe.py`: the translation module participates in the
  executable cache digest and is bundled by its normal Python import. The custom
  ICO is preserved by the updater. Local tests use the Python editor; rebuild the
  frozen EXE with `BUILD_FINAL_WINDOWS_EXE.bat` when preparing a player package.
- `crates/rc-game/src/weapon_evo.rs`: optional Bomb/Pyro capacities and mode-dependent
  effective registry values for other tracks. `experimental_ammo_enabled` mirrors
  the schema switches. Older profiles have no alternate and use native fallbacks.
- Capability **55**, patch **0.1.6.55**, launcher **0.15.0**. Older runtimes are rejected
  when a profile contains the new key, even if its experiment is currently Off.
- Save format remains WEVO-v3 from v54. This update adds no new save fields.

## Localized weapon-name references

The catalogs use names of the original 2002 game, rather than literal translations
or names from its 2016 reimagining. Community reference lists were checked on
2026-10-21. These sources document the localized names; they are not official
translations of this mod's newly authored interface/help text.

- [English weapons](https://ratchetandclank.fandom.com/wiki/Category:Weapons_in_Ratchet_%26_Clank_(2002_game))
- [Italian original-game weapons](https://it.wikipedia.org/wiki/Ratchet_%26_Clank_(videogioco_2002)#Armi)
- [French original-game list](https://ratchet.fandom.com/fr/wiki/Liste_des_armes)
- [German original-game list](https://ratchetandclank.fandom.com/de/wiki/Waffenliste)
- [Spanish original-game list](https://ratchetandclank.fandom.com/es/wiki/Lista_de_armas)

Examples: R.Y.N.O. / M.I.P.S. / T.E.L.T. / R.I.D.A. / T.A.U.N.; Pyrocitor /
Inceneritore / Pyrolanceur / Pyrozitor / Piromátic. Game-language changes still use
the disc's names in the actual game; launcher language changes only the editor.

## Verification and in-game checklist

Python fixtures cover every grade's ordering, both color groups, missing-capacity
migration, explicit overrides, boundaries, runtime capability rejection, all five
catalogs, actual dropdown callbacks and language switches with dirty documents.
Rust fixtures cover independent grade values, mode selection, old-profile fallbacks
and invalid capacities. Existing combat/XP, save, projectile and updater tests remain.

On Windows, run the local kit's first BAT to update/build, then open the test editor
with its second BAT. Test a standard and experimental grade with different capacities,
collect ammo and compare HUD/paused inventory; test Suck Cannon's held objects too.
Change language with unsaved RGB/damage edits and confirm the values remain. Save,
restart, and confirm the language and selected preset persist. Check all five grade
cards, the two global frames and the preserved custom window icon. Windows engine
compilation, real Tk layout and disc-data gameplay need local validation.
