# Flame-mine size for every selected model

Runtime patch 0.1.6.59 keeps profile capability 58 and the existing
`flame_mine_size_percent` key. The public mod version remains 0.3.

The size control applies to every deployed experimental Mine Glove model:

| Model | Percentage reference |
| --- | --- |
| Native mine | Existing native deployed-mine size |
| Bomb Glove bomb | Selected bomb mesh's native class scale |
| Glove of Doom sphere | Selected sphere mesh's native class scale |

100 preserves the selected model's reference size, 150 multiplies it by 1.5,
and 200 doubles it. This is a linear size multiplier, not a volume multiplier.
The existing default remains 150 for all five grades; no balance value changes.

## Implementation

`mine::experimental::snapshot_configured` captures the percentage at release.
`mine::experimental::visual_scale` computes alternative appearance directly as
`selected_model_native_scale * captured_percent / 100`. Native mines retain
their existing deployed physical scale. The engine's `gameplay::upload_dynamic`
uses this result for both the instance transform and visibility sphere.

This explicit path avoids deriving a replacement mesh's size through the
physical mine's held-object/release scale and class-conversion ratio. Alternative
meshes cannot fall back to a class-sized transform that omits their percentage.
The value remains attached to that mine after equipment/grade changes and is
applied once, including when a snapshot is repeated.

The physical object remains a Mine Glove mine with item-17 damage and XP. The
existing percentage-based clearance/landing probes remain active; model choice
does not spawn robots, alter the flame damage/range or select another weapon's
configuration. Colors, mine lifetime, flame rotation and sound ownership retain
their independent snapshots.

## Verification

The game-library regression covers all three model choices at 50, 100, 150,
200 and 500 percent, distinct native model scales, repeated snapshots and an
actual flight update. It checks visible scale, unchanged physics class and
captured percentage. The Windows workflow compiles the consuming engine path.

In game, compare newly deployed 100% and 200% bombs/spheres with the same grade,
then repeat using native mines. Change grade while older mines remain deployed;
existing mines must retain their captured size. Check landing and clearance near
floors and walls separately from model size. Runtime --version-json must report
0.1.6.59 when testing this correction.
