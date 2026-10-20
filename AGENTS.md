# Weapon Evolution development rules

This is an independent mod for ReRAC 0.3.4-alpha. Keep the distinction between
the mod author, the ReRAC developers and the original game's developers clear.

- Write product strings, configuration comments and developer documentation in English.
- Preserve existing balance, colors, XP requirements, presets and save data.
- In every weapon grade, place **all standard settings first**. Place experimental
  on/off switches and all their parameters **after every standard setting**, in
  both the INI and launcher. Apply this to every future experiment too.
- Register experimental fields in `launcher/schema.py`'s `EXPERIMENTAL_DEFAULTS`.
  The shared migration and `field_groups()` enforce the ordering. Add matching
  runtime consumers, validation and per-grade regression coverage.
- Snapshot projectile-specific damage and contents at creation/fire time. One
  projectile/explosion may damage a given enemy once; another instance may hit immediately.
- Keep damage/XP clocks independent of reaction cooldowns. Rapid-fire reaction
  limits must never suppress accepted HP loss. Respect native phase shields.
- Do not reintroduce temporary audio probes or record audio by default.
- Keep immutable level-table recognition on the loader worker. Use the explicit
  destination level index there; never change the active-level global to warm a cache.
- Do not commit executables, build outputs, disc data, personal paths or author settings.
