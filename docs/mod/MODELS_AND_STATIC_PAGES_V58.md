# Local v58: model palettes and withdrawn persistent editor pages

Gameplay patch 0.1.6.58, configuration capability 58.

**Launcher correction:** the persistent-page implementation below was withdrawn in launcher 0.17.1 after incomplete rendering on Windows. [v58.1 restores the original page lifecycle](LAUNCHER_RESTORE_V58_1.md). The model and steering sections below still describe current gameplay.
This source test is unpublished and includes the complete v57 fixes.

**Model size correction:** runtime 0.1.6.59 applies the flame-mine percentage directly to every selected model; see [FLAME_MINE_MODEL_SCALE.md](FLAME_MINE_MODEL_SCALE.md).

## Persistent editor

`Launcher._page_cache` owns complete page frames, sharing the document's
StringVars. Navigation switches the Canvas window to an existing frame. Scroll
only changes the Canvas view; it never constructs fields. Each page retains its
scroll position. Remaining pages are prepared during short idle-time tasks.
A new layout is constructed outside the visible Canvas area at its real width,
then attached after geometry calculation. There is no partially built page.

Cache keys include page, search text and all five model selectors. Normal edits
reuse existing widgets. Model changes and filtering replace only the affected
layout; hidden palette values remain in the document. Cached color swatches and
dropdown display variables refresh when presented. Loading another document or
changing language clears the cache and cancels pending page/wheel jobs.
Resize rearranges existing grade cards. Wheel events are coalesced every 12 ms;
Off/On and other dropdown values still cannot change through mouse-wheel input.

On eligible Windows Canvas windows, `enable_composited_paint` requests
WS_EX_COMPOSITED descendant buffering. It preserves other styles and rejects
CS_OWNDC/CS_CLASSDC/CS_PARENTDC, as required by Microsoft's
[extended window style documentation](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles).
Other platforms retain the same persistent/offscreen page behavior without the
Windows flag. This source environment has no Windows desktop: final visual
smoothness must be checked on the target computer.

## Model codes and ownership

| Per-grade key | Code | Appearance |
| --- | ---: | --- |
| Visibomb `grenade_model` | 0 | Bomb Glove bomb (existing default) |
| | 1 | Mine Glove mine |
| | 2 | Native Visibomb missile |
| Mine Glove `flame_mine_model` | 0 | Native mine (existing default) |
| | 1 | Bomb Glove bomb |
| | 2 | Glove of Doom glowing canister sphere |

Selectors appear immediately before experimental colors. The editor shows only
the selected model's relevant palette, in all five languages. Stable numeric
codes are stored in the INI. Off/On switches retain canonical false/true values.

These are **visual overrides** in `gameplay::upload_dynamic`, not physics class
changes. Visibomb grenades remain class 121 with item-13 HP/XP ownership; flame
mines remain class 74 with item-17 ownership. Collision, fuse, active-mine limits,
ammo consumption, sounds and existing damage attribution do not depend on the
visual choice. The renderer compensates for the selected class's native scale
and uses its own neutral pose instead of the original class's animation state.
Missing reference graphics are imported on the existing loader worker using its
explicit destination level; no disc reads occur during firing or rendering.

Visibomb model 2 follows its velocity with roll zero, yaw along horizontal travel
and pitch from vertical travel. This runs even when its tint is `default` and
after gravity changes the trajectory. Other models retain normal bomb rotation.
No missile camera or remote-control behavior is activated for these grenades.

## Independent palettes

Visibomb:

- Bomb: `grenade_color`, `grenade_explosion_color` (existing values retained).
- Mine: `grenade_mine_color`, `grenade_mine_explosion_color`.
- Missile: `grenade_missile_color`, `grenade_missile_explosion_color`.

Flame Mine:

- Mine: `mine_model_color`, existing `flame_core_color`, `flame_outer_color`,
  `burn_flame_color`.
- Bomb: `mine_bomb_color`, `mine_bomb_core_color`, `mine_bomb_outer_color`,
  `mine_bomb_burn_color`.
- Sphere: `mine_sphere_color`, `mine_sphere_core_color`,
  `mine_sphere_outer_color`, `mine_sphere_burn_color`.

Each active projectile snapshots its model and chosen colors when created.
Changing equipment/grade or editing another model's palette cannot recolor an
already deployed projectile. `default` uses native colors without borrowing
another model's settings. Visual overrides also suppress the physical bomb's
animated glow when using a different model with native colors. Explicit body
tints cover that model's emissive packets too. Standard mine explosion/light controls retain their
existing shared semantics; model-specific flame colors remain experimental.

Migration adds 75 missing entries across five grades, defaults both selectors to
0 and changes no v57 values. Presets, comments, BOM and line endings are retained.
The new fields require capability 58 even with experiments Off, preventing older
runtimes from silently ignoring their configuration.

## Guided native Visibomb fix

The previous launch timer scaled with `duration_multiplier`, but steering,
launch-ground clearance and explosion timing compared it to a fixed 3000 ticks.
With duration 2, steering therefore stayed disabled for almost the first entire
native flight duration. The initial lifetime and `turn_multiplier` now belong to
the missile's captured extended pvars. All elapsed-flight checks use that initial
lifetime. A longer flight retains the native short steering warmup, and larger
turn settings affect both yaw and pitch. Grenade mode remains independent.

## Verification and in-game checks

Automated tests cover page lifetimes/reuse, offscreen construction, warmup,
scroll routing, retained values, model filtering and order, all five languages,
lossless profile migration, Rust model validation/snapshots, native missile
steering with duration 2 and triple turn strength, and ascending/descending rocket
orientation. The full pre-existing gameplay and launcher suites are also run.

The test kit runs the Rust suite and compiles the actual Windows engine locally.
No Windows binary or disc gameplay is claimed to have been tested here.

On Windows, check navigation/scrolling immediately after startup and again after
all pages are warm. Switch language, resize, load another preset, copy a model
selection to other grades and restore a saved value. Verify that no hidden colors
are lost. Test all three grenade/mine models in a level that does not normally
contain their graphics. Check rising/falling missile orientation, collision,
water/death-pit behavior, independent flames/XP and the existing v57 death cleanup.
