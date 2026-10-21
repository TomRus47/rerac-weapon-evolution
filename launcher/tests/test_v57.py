"""Mouse-wheel routing must never invoke Tk's dropdown selection binding."""
from pathlib import Path
import sys
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import launcher as gui


class WheelTests(unittest.TestCase):
    def test_test_opener_rejects_a_stale_runtime_with_the_same_configuration_capability(self):
        import importlib.util
        path = Path(__file__).resolve().parents[2] / 'tools/mod/local_test_update.py'
        spec = importlib.util.spec_from_file_location('v57_test_updater', path)
        updater = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(updater)
        with patch.object(updater.core, 'probe_exe', return_value={'weapon_evolution_patch': '0.1.6.56'}), \
                patch.object(updater.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'v59 runtime'):
                updater.open_editor(Path('test-mod'))
            run.assert_not_called()

    def test_dropdown_and_open_list_class_bindings_stop_selection_and_scroll_once(self):
        app = gui.Launcher.__new__(gui.Launcher)
        app.root = Mock()
        app.canvas = Mock()
        app.bind_mousewheel()
        self.assertEqual(app.root.bind_all.call_count, 3)
        self.assertEqual(app.root.bind_class.call_count, 6)
        for call in app.root.bind_class.call_args_list:
            name, sequence, callback = call.args
            self.assertIn(name, ('TCombobox', 'ComboboxListbox'))
            event = SimpleNamespace(num=4 if sequence == '<Button-4>' else 5 if sequence == '<Button-5>' else None,
                                    delta=-120 if sequence == '<MouseWheel>' else 0)
            app.canvas.reset_mock()
            self.assertEqual(callback(event), 'break')
            app.canvas.yview_scroll.assert_called_once_with(-1 if event.num == 4 else 1, 'units')

    def test_form_wheel_ignores_zero_delta_and_normalizes_trackpad_input(self):
        app = gui.Launcher.__new__(gui.Launcher)
        app.root = Mock()
        app.canvas = Mock()
        app.form = object()
        app.root.winfo_containing.return_value = SimpleNamespace(master=app.form)
        for delta, expected in [(120, -1), (-240, 2), (1, -1), (-1, 1)]:
            app.canvas.reset_mock()
            self.assertEqual(app.mousewheel(SimpleNamespace(delta=delta, x_root=1, y_root=2)), 'break')
            app.canvas.yview_scroll.assert_called_once_with(expected, 'units')
        app.canvas.reset_mock()
        app.dropdown_mousewheel(SimpleNamespace(delta=0))
        app.canvas.yview_scroll.assert_not_called()
