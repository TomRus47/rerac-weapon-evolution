from pathlib import Path
import shutil,struct,sys,tempfile,unittest
from types import SimpleNamespace
from unittest.mock import patch,Mock
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import build_launcher_exe as b
import launcher as gui

def ico(payload=b'abcd'):
    return struct.pack('<HHH',0,1,1)+struct.pack('<BBBBHHII',16,16,0,0,1,32,len(payload),22)+payload

class IconTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='custom icon ');self.addCleanup(self.tmp.cleanup)
        self.kit=Path(self.tmp.name).resolve()
        for name in b.SOURCES:shutil.copy2(ROOT/name,self.kit/name)
    def test_icon_is_optional_and_changing_or_removing_it_invalidates_the_cache(self):
        self.assertIsNone(b.icon_file(self.kit));before=b.source_digest(self.kit)
        p=self.kit/'launcher.ico';p.write_bytes(ico())
        first=b.source_digest(self.kit);self.assertNotEqual(first,before)
        p.write_bytes(ico(b'new!'));self.assertNotEqual(b.source_digest(self.kit),first)
        p.unlink();self.assertEqual(b.source_digest(self.kit),before)
    def test_renamed_images_and_incomplete_ico_directories_are_rejected(self):
        p=self.kit/'launcher.ico'
        for raw in [b'\x89PNG\r\n\x1a\n',b'JPEG image',struct.pack('<HHH',0,1,1),ico()[:-1]]:
            p.write_bytes(raw)
            with self.assertRaises(ValueError):b.icon_file(self.kit)
    def test_builder_embeds_the_icon_for_both_windows_and_tk(self):
        p=self.kit/'launcher.ico';p.write_bytes(ico());calls=[]
        def freeze(args,**kwargs):
            calls.append(args);out=Path(args[args.index('--distpath')+1]);out.mkdir(parents=True)
            (out/b.EXE_NAME).write_bytes(b'compiled exe')
        with patch.object(sys,'platform','win32'),patch.object(b,'ensure_tools',return_value=self.kit/'python.exe'),patch.object(b.subprocess,'run',side_effect=freeze),patch.object(b,'smoke_test',return_value={'gui_ok':True}),patch.object(b,'collect_licenses',side_effect=lambda py,out:out.mkdir()):
            b.build(self.kit)
        args=calls[0]
        self.assertEqual(args[args.index('--icon')+1],str(p))
        self.assertEqual(args[args.index('--add-data')+1],str(p)+':.')
    def test_window_icon_uses_the_bundled_copy_in_a_frozen_exe(self):
        bundle=self.kit/'unpacked';bundle.mkdir();(bundle/'launcher.ico').write_bytes(ico())
        root=Mock()
        with patch.object(sys,'platform','win32'),patch.object(sys,'_MEIPASS',str(bundle),create=True):
            gui.apply_window_icon(root,SimpleNamespace(TclError=RuntimeError),self.kit)
        root.iconbitmap.assert_called_once_with(default=str(bundle/'launcher.ico'))
    def test_window_icon_is_optional_and_never_blocks_startup(self):
        root=Mock()
        with patch.object(sys,'platform','win32'):
            gui.apply_window_icon(root,SimpleNamespace(TclError=RuntimeError),self.kit)
        root.iconbitmap.assert_not_called()
        (self.kit/'launcher.ico').write_bytes(ico());root.iconbitmap.side_effect=RuntimeError('unsupported')
        with patch.object(sys,'platform','win32'):
            gui.apply_window_icon(root,SimpleNamespace(TclError=RuntimeError),self.kit)

if __name__=='__main__':unittest.main()
