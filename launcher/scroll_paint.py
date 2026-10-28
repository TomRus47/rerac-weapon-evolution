"""Keep scroll input and repainting together without rebuilding editor pages.

Canvas window items contain native editor windows. After moving them, Windows
needs the complete viewport subtree invalidated, including the unchanged area
which Canvas may otherwise preserve by copying old pixels. Do not change window
styles, suspend redraw globally, reparent controls, or dispatch nested input.
"""
import sys


def windows_redraw():
    if sys.platform != 'win32':
        return None
    import ctypes
    from ctypes import wintypes
    function = ctypes.WinDLL('user32', use_last_error=True).RedrawWindow
    function.argtypes = (wintypes.HWND, ctypes.c_void_p, wintypes.HRGN, wintypes.UINT)
    function.restype = wintypes.BOOL
    return function


class ViewportPainter:
    def __init__(self, canvas, redraw=None):
        self.canvas = canvas
        self.redraw = windows_redraw() if redraw is None else redraw

    def paint(self):
        # Flush geometry first so that child HWNDs have their final positions.
        # update_idletasks does not dispatch clicks, key presses or wheel input.
        self.canvas.update_idletasks()
        if self.redraw is not None and self.canvas.winfo_ismapped():
            # RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW.
            # WM_PAINT can queue Tk display work; flush that work before return.
            flags = 0x0001 | 0x0004 | 0x0080 | 0x0100
            if not self.redraw(self.canvas.winfo_id(), None, None, flags):
                raise RuntimeError('Windows could not redraw the launcher viewport.')
            self.canvas.update_idletasks()


class ScrollBatch:
    """One bounded scroll transaction per frame; all controls already exist."""
    def __init__(self, canvas, painter, interval_ms=16):
        self.canvas = canvas
        self.painter = painter
        self.interval_ms = interval_ms
        self.pending = []
        self.job = None
        self.closed = False
        self.flushing = False

    def request(self, *command):
        if self.closed or not command:
            return
        if command[0] == 'moveto':
            # A scrollbar drag replaces older positions that were never shown.
            self.pending = [command]
        elif (self.pending and len(command) == 3 and command[0] == 'scroll'
              and self.pending[-1][0] == 'scroll' and self.pending[-1][2] == command[2]):
            previous = self.pending[-1]
            # Keep opposite movements separate: hitting a boundary and moving
            # back is different from cancelling their numeric sum.
            if int(previous[1]) * int(command[1]) >= 0:
                self.pending[-1] = ('scroll', int(previous[1])+int(command[1]), command[2])
            else:
                self.pending.append(command)
        else:
            self.pending.append(command)
        if self.job is None and not self.flushing:
            self.job = self.canvas.after(self.interval_ms, self.flush)

    def flush(self):
        if self.closed or self.flushing:
            return
        if self.job is not None:
            self.canvas.after_cancel(self.job)
            self.job = None
        pending, self.pending = self.pending, []
        if not pending:
            return
        self.flushing = True
        try:
            for command in pending:
                self.canvas.yview(*command)
            self.painter.paint()
        finally:
            self.flushing = False
            if self.pending and not self.closed and self.job is None:
                self.job = self.canvas.after(self.interval_ms, self.flush)

    def close(self):
        self.closed = True
        if self.job is not None:
            self.canvas.after_cancel(self.job)
            self.job = None
        self.pending.clear()
