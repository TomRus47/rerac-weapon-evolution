# Local v57 fixes

This test patch is based on public Weapon Evolution 0.3 (gameplay v56).
It is not published. Runtime patch: 0.1.6.57; launcher: 0.16.1. Configuration
capability remains 56 because no profile fields or save formats are added.

## Inventory and experimental capacities

The engine previously clamped actual stock against standard `max_ammo` while
the HUD and pickups used the experimental capacity. With stock 40 and an active
experimental cap of 5, the HUD could stay at 5 for many accepted shots.

`ExtraTrack::ammo_capacity` now selects the active standard/experimental cap.
Both `publish_max_ammo` and `Evolution::apply_ammo_limits` consume that selection.
The engine calls `apply_ammo_limits` after restoring inventory and before weapon
updates. Bomb Glove and Pyrocitor use their existing typed `ammo_capacity`
selection. This only constrains stock: switching modes or grades never refills
it. Stream ammunition intervals and damage clocks remain independent.

Suck Cannon stores actual moby objects and uses its existing `capacity` queue
limit. It is intentionally excluded from ordinary inventory-counter clamping.

## Flame-mine lifetime and terrain contact

`Services::reset_moby_transients` runs after successful death-reload preflight,
before the replacement moby table/load pass. It now frees all dynamic light
slots owned by that old table, retaining allocation generations. Renderer world
light lists can therefore detach old lights even if a slot is immediately reused.

The mine flight solver previously flipped normals against velocity. A 200%
mine can overlap a floor while rising, because its hull is wider than the native
release clearance. That separating contact could reflect it downward and place
it underneath the floor. `contact_normal` now chooses the previous centre's
side. `departing_surface` resolves penetration outward while retaining launch
velocity. Incoming floor/wall contacts still land or bounce normally.

The shared glove update also retains a mine release request when a dynamic slot
cannot yet be allocated. A mine consumes ammunition and emits throw feedback
only when its actual object is released. Bomb/Doom/Decoy timing is unchanged.
The synthetic allocator test covers a full table and its two-tick reuse delay.
The reported Kalebo III checkpoint itself still requires a game-data playtest.

## Launcher wheel routing

`Launcher.bind_mousewheel` replaces wheel bindings for `TCombobox` and the
`ComboboxListbox` popdown class. `dropdown_mousewheel` scrolls the form and
returns `break` before native selection changes or a second all-tag scroll.
Mouse-wheel and button-4/button-5 events share `scroll_page`; click and keyboard
selection bindings are preserved. Header language/preset dropdowns are covered.

Regression coverage is in `launcher/tests/test_v57.py`, the glove allocation
tests, `weapon_evo::v57_inventory_tests`, and mine experimental tests. See
[LOCAL_TEST_KIT.md](LOCAL_TEST_KIT.md) for installation and in-game checks.

Validation: 1,017 Rust game-library release tests and 122 Python tests passed.
Source integrity checks passed. Public INI/default/example values and custom
icon are unchanged. Windows compilation, real Tk rendering and the reported
Kalebo III location remain local gameplay checks.
