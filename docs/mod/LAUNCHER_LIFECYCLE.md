# Launcher page and build lifecycle

Launcher 0.29.3 builds all 17 pages before showing the main window. A slower
startup is intentional; navigation must not construct a partially visible page.
This supersedes the temporary v58 caching approach and its later restoration.

`page_views.py` owns each page's frame, Canvas, embedded form, rows and scroll
state. The form remains a child of its own Canvas for its whole lifetime. Never
reparent an embedded native window between Canvases. Configure callbacks retain
their owning view instead of following the mutable active-page aliases.

`CompactFieldRow` draws labels and color swatches on a Canvas while retaining
native entries and selectors. This reduces Windows handle use without changing
the profile model. Canonical `StringVar` values survive page switches and preset
loads. Model palettes are built once and shown/hidden; hiding one must not erase
its configuration. Language changes rebuild the generation while preserving
dirty values, selected preset and active game session.

`scroll_paint.py` combines rapid scroll requests into a 16-ms batch. It preserves
opposite-direction ordering, flushes the old page before navigation and cancels
callbacks when a page is destroyed. Windows repaint invalidates the viewport and
its native children together. Unchanged width and row geometry are cached.
Do not add global redraw suppression, reentrant input dispatch or composited
window styles: those caused missing or corrupted controls in earlier attempts.
Native Windows scrolling still needs visual playtesting on different systems.

`page_smoke.py` exercises every page, all eight grades, five languages, three
column layouts, palettes, scroll positions and unchanged widget identities.
The frozen builder requires its preload, repaint and experimental-override
proof flags. Handle checks protect the Windows per-process USER/GDI budgets.
The supplied author-PC proof passed with 6,229 USER objects and 52 GDI objects;
these are one run's measurements, not guaranteed performance figures.

`build_lifecycle.py` distinguishes a failed self-test from temporary Windows
file locks. Only file-access operations receive bounded retries. A failed GUI
proof is never accepted or retried as success. Keep its JSON, progress, traceback
and process log outside the temporary build tree; retain a failed stage for
diagnosis. Cleanup after success must not overwrite the original build result.

For field changes, update schema, migration, runtime consumer, localization,
eight-grade tests and smoke coverage together. See
[alternate-mode snapshots](EXPERIMENTAL_OVERRIDES_V109.md) and the
[configuration reference](CONFIG_REFERENCE.md).
