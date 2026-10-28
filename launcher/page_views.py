"""Persistent editor pages with fixed Canvas ownership and compact field rows.

Tk work stays on the UI thread. A page owns its Canvas, scrollregion, grade grid
and every field for its entire document generation. No Canvas window is moved
between parents and no background/idle task creates controls during navigation.
"""
from __future__ import annotations
from scroll_paint import ScrollBatch, ViewportPainter


class PageView:
    def __init__(self, app, key, parent):
        self.key = key
        self.frame = app.ttk.Frame(parent)
        self.frame.grid(row=0, column=0, sticky='nsew')
        self.canvas = app.tk.Canvas(self.frame, highlightthickness=0, bg='#edf0f5')
        self.painter = ViewportPainter(self.canvas)
        self.scrolling = ScrollBatch(self.canvas, self.painter)
        self.scrollbar = app.ttk.Scrollbar(self.frame, orient='vertical', command=self.scroll)
        self.scrollbar.pack(side='right', fill='y')
        self.canvas.pack(side='left', fill='both', expand=True)
        self.canvas.configure(yscrollcommand=self.scrollbar.set)
        self.form = app.ttk.Frame(self.canvas)
        self.window = self.canvas.create_window(0, 0, window=self.form, anchor='nw')
        self.cards = []
        self.area = None
        self.columns = 0
        self._width = None
        self._region = None
        try:
            self.card_width=max(355,round(355*float(self.canvas.winfo_fpixels('1i'))/96.))
        except (TypeError,ValueError):
            self.card_width=355
        self.rows = []
        self.filtered_cells = []
        self.form.bind('<Configure>', lambda _: self.update_region())
        self.canvas.bind('<Configure>', lambda event: self.resize(event.width))
        self.frame.bind('<Destroy>', self.destroyed, add='+')

    def scroll(self, *command):
        self.scrolling.request(*command)

    def destroyed(self, event):
        if event.widget is self.frame:
            self.scrolling.close()

    def update_region(self):
        region = self.canvas.bbox('all')
        if region != self._region:
            self.canvas.configure(scrollregion=region)
            self._region = region

    def resize(self, width):
        if width == self._width:
            return
        self._width = width
        self.canvas.itemconfigure(self.window, width=max(1, width))
        columns = max(1, min(5, width // getattr(self,'card_width',355)))
        if columns != self.columns:
            self.columns = columns
            self.arrange()

    def arrange(self):
        if self.area is None:
            return
        for column in range(5):
            self.area.columnconfigure(column, weight=0, uniform='')
        columns = self.columns or 2
        for column in range(columns):
            self.area.columnconfigure(column, weight=1, uniform='grades')
        for index, card in enumerate(self.cards):
            card.grid(row=index // columns, column=index % columns,
                      sticky='new', padx=5, pady=5)

    def refresh(self, app):
        # Sync only presentations. Canonical StringVars remain the single source
        # of truth, including values in hidden palettes and offscreen grades.
        self.scrolling.flush()
        for row in self.rows:
            row.sync()
        parents = {}
        for row in self.rows:
            parents.setdefault(row.parent, []).append(row)
        for rows in parents.values():
            wanted = [row.matches(app) for row in rows]
            for row, show in zip(rows, wanted):
                if row.visible and not show:
                    row.canvas.pack_forget()
                    row.visible = False
            for index in reversed(range(len(rows))):
                row = rows[index]
                if wanted[index] and not row.visible:
                    following = next((r.canvas for r in rows[index+1:] if r.visible), None)
                    options = {'fill': 'x', 'pady': 5}
                    if following is not None:
                        options['before'] = following
                    row.canvas.pack(**options)
                    row.visible = True
        for widget, pair in self.filtered_cells:
            if app.matches(*pair):
                widget.grid()
            else:
                widget.grid_remove()


class FieldRow:
    """One native row plus its editor; labels/swatches do not allocate widgets.

    The old frame + label + cell + editor (+ color button) hierarchy multiplied
    native-window usage when all pages were retained. Canvas text and a clickable
    swatch preserve that presentation with substantially fewer native controls.
    """
    def __init__(self, app, parent, section, key, label):
        self.app, self.parent, self.pair = app, parent, (section, key)
        self.canvas = app.tk.Canvas(parent, bg='#ffffff', highlightthickness=0,
                                    bd=0, width=300, height=34)
        self.canvas.pack(fill='x', pady=5)
        self.visible = True
        self.editor = None
        self.window = None
        self.swatch = None
        self._layout_shape = None
        self.sync = lambda: None
        self.label = self.canvas.create_text(0, 1, anchor='nw', text=label,
            fill='#202939', font=('Segoe UI', 10), width=180)
        self.hint = None
        self.canvas.bind('<Configure>', lambda event: self.layout(event.width))

    def attach(self, editor, hint=None):
        self.editor = editor
        self.window = self.canvas.create_window(298, 1, window=editor, anchor='ne')
        if hint:
            self.hint = self.canvas.create_text(0, 35, anchor='nw', text=hint,
                fill='#627088', font=('Segoe UI', 8), width=290)

    def color_action(self, command):
        self.swatch = self.canvas.create_rectangle(277, 2, 299, 24,
            fill='#b4bdcc', outline='#b4bdcc', tags=('swatch',))
        self.canvas.tag_bind('swatch', '<Button-1>', lambda _: command())
        self.canvas.tag_bind('swatch', '<Enter>', lambda _: self.canvas.configure(cursor='hand2'))
        self.canvas.tag_bind('swatch', '<Leave>', lambda _: self.canvas.configure(cursor=''))

    def paint(self, color):
        if self.swatch is not None:
            self.canvas.itemconfigure(self.swatch, fill=color)

    def layout(self, width):
        if self.editor is None:
            return
        shape = (width, self.editor.winfo_reqwidth(), self.editor.winfo_reqheight())
        if shape == self._layout_shape:
            return
        self._layout_shape = shape
        right = max(1, width) - 1
        if self.swatch is not None:
            self.canvas.coords(self.swatch, right-22, 2, right, 24)
            right -= 28
        self.canvas.coords(self.window, right, 1)
        editor_width = shape[1]
        self.canvas.itemconfigure(self.label, width=max(60, right-editor_width-10))
        bounds = self.canvas.bbox(self.label)
        height = max(shape[2]+2, bounds[3]+2 if bounds else 30)
        if self.hint is not None:
            self.canvas.coords(self.hint, 0, height+3)
            self.canvas.itemconfigure(self.hint, width=max(60, width-2))
            bounds = self.canvas.bbox(self.hint)
            height = bounds[3]+2 if bounds else height+35
        if int(float(self.canvas.cget('height'))) != height:
            self.canvas.configure(height=height)

    def matches(self, app):
        from schema import visible_model_field
        section, key = self.pair
        weapon = section.split('.')[1] if section.startswith('weapon.') else ''
        selector = 'grenade_model' if weapon == 'visibomb' else 'flame_mine_model'
        model = app.variables.get((section, selector))
        return app.matches(section, key) and visible_model_field(
            weapon, key, model.get() if model else '0')
