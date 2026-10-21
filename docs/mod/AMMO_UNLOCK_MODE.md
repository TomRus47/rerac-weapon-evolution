# Ammunition-based arsenal (experimental, v52)

Enable **Experimental ammo-unlock mode** under **Global Settings** in launcher
0.12.0, or set this in the public `weapons.ini`:

```ini
[general]
experimental_ammo_unlock_mode = true
```

The default is `false`. This requires the v52 / configuration-capability 52
runtime. It is independent of per-weapon experimental firing modes. Gold
purchases retain their normal meaning outside this mode.

## Ownership and quick select

An ammo weapon is available while it has at least one round. A collected pack
adds the normal pickup quantity, capped at that grade's `max_ammo`. Spending
the final round removes ownership and its quick-select entry; existing shots,
mines, robots or companions may finish their normal lifetime. Collecting ammo
again restores ownership and inserts the icon again. A locked hand item
requests the OmniWrench using the normal item-swap path. A last-round Visibomb
finishes its guided flight before that forced swap; its ownership and ring
entry still lock immediately.

The ring still has eight slots. An unlock uses an empty slot first; a full
ring replaces a slot in rotating order, avoiding the equipped item. It cannot
display the entire fifteen-weapon arsenal simultaneously.

The OmniWrench remains available. The four ammo-free weapons unlock at these configurable collected-bolt totals:

| Weapon | Collected normal bolts |
|---|---:|
| Taunter | 10,000 |
| Suck Cannon | 25,000 |
| Walloper | 50,000 |
| Morph-o-Ray | 75,000 |

These are **collected** normal bolts, not wallet balance, Gold Bolts or XP.
Spending bolts does not reverse an unlock. Actual bolt pickups and the native
bolt-pool overflow credit count; arbitrary wallet awards and refunds do not.
When enabling the mode in an existing save, its saved per-planet collection
statistics seed the total. New games created with the mode enabled start with zero Bomb Glove ammo and the OmniWrench. Native planet-entry grants cannot refill or equip the locked glove. Existing ammunition remains when enabling the mode on an existing save; loading or changing planets never clears collected rounds. Subsequent ordinary story weapon grants keep their
story-acquired flag and their ownership for restoration, without granting free
weapon ammo or bypassing the mode's unlock rule. Utility gadgets remain native.

## Ammo crates and rarity

Each ammo crate drops **one to four normal ammo packs**, not one to four single
rounds. Every pack picks its weapon independently, so repeats are possible.
All eleven ammo weapons are eligible immediately, including unowned weapons.
Placed fixed-type ammo crates are randomized too. Loose placed packs keep their
original type. Each pack retains its weapon's native pickup amount; a disc
with no R.Y.N.O. pickup amount uses one round rather than an empty pack.

| Gold Bolts collected | 1 pack | 2 packs | 3 packs | 4 packs |
|---|---:|---:|---:|---:|
| 0 | 50% | 40% | 8% | 2% |
| 20 | 37.5% | 32.5% | 16.5% | 13.5% |
| 40 or more | 25% | 25% | 25% | 25% |

For each Gold Bolt up to 40, the probabilities change by **-0.625, -0.375,
+0.425, +0.575 percentage points**, respectively. Integer tickets make these
probabilities exact; there is no floating-point drift or extra reward for
spent versus unspent Gold Bolts.

| Ammo weapon | Relative selection weight |
|---|---:|
| Bomb Glove, Blaster, Pyrocitor | 30 each |
| Devastator, Mine Glove, Tesla Claw, Glove of Doom, Drone Device | 10 each |
| Visibomb Gun, Decoy Glove | 3 each |
| R.Y.N.O. | 1 |

These are the default ratios (total 147). v52 exposes all eleven normalized percentage inputs in Global Settings and `ammo_drop_chance_<weapon>` under `[general]`. Set zero to exclude a type; at least one input must be positive. The launcher shows the normalized actual chances. Default inputs preserve these ratios. Weapons at full ammo remain eligible in this mode, so
ownership or ammo capacity cannot remove the intended rarity. Packs obey the
normal physical pickup and inventory-capacity rules.

Outside this mode the native fixed crate types and one/two-pack count remain.
For randomized crates, owned R.Y.N.O. ammo can now appear even when the US
vendor list omits it. The usual owned/low-ammo preference remains; R.Y.N.O.
has weight 1 and other eligible weapons have weight 20 in ordinary play.

## Shops, save compatibility and implementation

While active, Gadgetron vendors and their holograms, Gold offers/props and the
R.Y.N.O. salesman are disabled. The PDA cannot open an ammo shop. Vendor stock
and purchase processing are also guarded. Story/gadget purchases are retained.
Disabling the mode and reloading the level restores those objects from the
level data; the disable path never writes their permanent death bits.

The mod companion format is now **WEVO version 2**. The native save layout is
unchanged. Version-1 XP companions migrate without losing weapon grades/XP.
Mode state includes collected bolts, available weapons, the native ownership
backup, the original quick-select ring and its replacement cursor. It uses the
same slot/content-addressed, checksummed, pre-commit companion write as XP.
Loading another slot or starting a new game cannot inherit another slot's mode
state. Disabling restores the saved native weapon ownership/ring, including
native story grants accepted while active. Saves written by v51 should be used
with v51 or newer; older mod runtimes do not understand a v2 companion.

Key integration points:

- `weapon_evo_ammo.rs`: deterministic ownership reconciliation, thresholds,
  integer pack probabilities and rarity; pure regression fixtures.
- `weapon_evo_save.rs`: backward-compatible v1 decoding and v2 mode persistence;
  mode changes do not reset live XP or combat epochs.
- `gameplay.rs`: reconcile before inventory mirrors and after accepted pickups,
  ammo use and story writes; drain actual bolt collection separately from cash.
- `pickup.rs` / `crate_.rs`: native pack spawn/collection and independent choices.
- `companion_assets.rs`: loader-worker imports of missing ammo models; the normal
  pickup update is explicitly installed even without a native per-level entry.
- Vendor/PDA/Gold-offer/R.Y.N.O.-seller entry points: no weapon/ammo purchase path.
- `launcher/schema.py` and `migrate_ini.py`: opt-in switch and lossless migration.

No extracted game assets are distributed. Model imports read the player's own
extracted disc on the loader worker, cache their reference data, and remap
texture indices without modifying shared native classes.

## Gold appearance and Tesla audio

`gold_texture` is now exposed for all fifteen evolving weapons in all five
grades, including Visibomb, Taunter, Walloper and R.Y.N.O. It uses the same
native Gold environment-map replacement as the Drone Device: metallic model
surfaces shine gold; decals, ordinary nonmetal textures and effect colors keep
their own settings. This is separate from `gold_effects` and from Gold ownership.
The OmniWrench has `wrench_gold_texture` in Global Settings because it has no
XP grades. Defaults for existing fields and the supplied custom icon remain.

Dual Tesla beams retain two independent native electrical chains and unchanged
damage/XP rules. Only the primary queues weapon sounds, including lock-on;
the outer firing update still owns the single hum and ammo clock.

## In-game review

Before publishing, test new and existing slots, every pack model, full ammo,
last-round expiry and re-unlock with a full ring, all three bolt thresholds,
mode-off restoration and all shop types. Test normal R.Y.N.O. loot after owning
the weapon. Inspect Gold materials in hand and pause previews on several
planets, and listen to dual Tesla acquisition with zero, one and two enemies.
Automated fixtures cover rules and persistence; they cannot establish the
player's disc asset coverage or subjective sound/render appearance.

## Local v52 configuration

The four `[general]` milestone fields are `ammo_unlock_taunter_bolts`, `ammo_unlock_suck_cannon_bolts`, `ammo_unlock_walloper_bolts`, and `ammo_unlock_morph_ray_bolts` (integer 0–1,000,000,000). They compare with lifetime collected normal bolts and do not spend currency. Changing the configuration recalculates availability from that total.

Global Settings contains exactly two cards: Base settings (general and HUD) followed by Experimental ammo-unlock mode, with its on/off switch first, four milestones next and eleven ammo probabilities last. Use the [local test kit](LOCAL_TEST_KIT.md) to rebuild and open the updated Python editor without recompiling its EXE or contacting GitHub.
