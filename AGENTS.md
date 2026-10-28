# Weapon Evolution development rules

This is an independent mod for ReRAC 0.3.7-alpha. Keep the distinction between
the mod author, the ReRAC developers and the original game's developers clear.

- Write code comments, configuration comments and developer documentation in English.
  Maintain localized launcher UI text in the shared language catalogs.
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

- Launcher UI is localized in English, Italian, French, German and Spanish.
  Keep code/docs/INI keys/comments in English. Add localized labels/help alongside
  new UI fields; preserve the selected language and dirty values when rebuilding UI.
- Use Off/On dropdowns for every boolean. Keep canonical false/true in profiles.
- Share field/section ordering between INI and editor. Colors always finish their
  own frame; Gold controls immediately precede standard colors. Register experiment
  enablers explicitly and keep runtime alternate-capacity selection in sync.

- Published README, release notes, configuration headers and player instructions must retain the mod-only authorship and original-game rights notice; see docs/mod/CREDITS.md.
