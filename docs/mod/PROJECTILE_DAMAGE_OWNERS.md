# Enemy projectile damage ownership

When `enemy_damage_by_health` is enabled, ordinary enemy damage uses the source
actor's original native HP tier and `enemy_damage_multiplier`, rounded to the
nearest whole health point with a minimum of one. The original tier corresponds
to 100 mod combat HP per native HP. It is independent of remaining HP and the
configurable enemy-health multiplier. NG+ applies its existing coefficient to
the effective damage multiplier. Attack-specific boss overrides retain priority.

Veldin II's tank (class 1454) initializes a six-HP numeric damage record at pvar
`+0x20`. It is outside the shared XP-donor catalog, which previously prevented
`Enemies::observe` from retaining its tier. Its shell (class 41) could then be
recorded with the generic one-HP fallback, including the native one-damage splash.
The damage-source registry now recognizes the tank's record independently of
XP eligibility. This change does not add the tank to the XP-donor catalog or
change its health, AI, projectile speed, collision, stun or visuals.

`veldin_tank::spawn_shell` explicitly snapshots the owner with
`Enemies::created_from` after allocation. This does not depend on the current
scheduler updater or the shell's class type. Both the direct hit and the beam
explosion continue to identify the shell as attacker and resolve the same saved
source tier. A source's later damage, deletion or slot reuse cannot alter a live
shot. Reusing the projectile slot clears its old snapshot normally.

For a tank shell, effective damage multipliers 0.5, 1, 2 and 2.5 therefore remove
3, 6, 12 and 15 Nanotech points, respectively. When health-class damage is Off,
the ordinary native hit amounts are used. This requires no new profile setting,
save migration or seed change.

The regression creates actual native shells with class types 0 and 5, a different
current updater, and a tank without a generic target-record header. It checks
initial and damaged source HP, direct/splash intake into Ratchet's actual health,
option gating, owner deletion/reuse and projectile-slot cleanup. Existing shared
registry and attack-specific boss regressions remain applicable.

Weapon Evolution is an independent mod by TomRus47. The author is not associated
with development of ReRAC or the original games and owns no original-game assets
or rights. Thanks to the ReRAC developers and contributors for their work.
