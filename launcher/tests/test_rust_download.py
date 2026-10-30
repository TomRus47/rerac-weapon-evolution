"""Only network failures select another supported Rustup transport."""
from pathlib import Path
import os,sys,unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools/mod'))
import rust_download as download

class RustDownloadTests(unittest.TestCase):
    def test_real_child_exit_and_bounded_stream_tail(self):
        code,detail=download._attempt([sys.executable,'-c',
            'import sys; [print("line",i) for i in range(120)]; sys.exit(7)'],dict(os.environ),ROOT)
        self.assertEqual(code,7);self.assertIn('line 119',detail)
        self.assertNotIn('line 0\n',detail);self.assertEqual(len(detail.splitlines()),100)

    def test_success_uses_one_attempt_with_original_proxy_and_dist_server(self):
        env={'PATH':'keep','HTTPS_PROXY':'existing-proxy','RUSTUP_DIST_SERVER':'https://static.rust-lang.org'}
        with patch.object(download,'_attempt',return_value=(0,'')) as attempt:
            download.install_toolchain('rustup','stable',env,ROOT)
        self.assertEqual(attempt.call_count,1)
        args,child,_=attempt.call_args.args
        self.assertEqual(args,['rustup','toolchain','install','stable','--profile','minimal'])
        for key,value in env.items():self.assertEqual(child[key],value)
        self.assertNotIn('RUSTUP_MAX_RETRIES',env)

    def test_connection_reset_uses_curl_once_and_does_not_mutate_parent(self):
        env={'PATH':'keep'}
        with patch.object(download,'_attempt',side_effect=[(1,'error downloading file: os error 10054'),(0,'')]) as attempt:
            download.install_toolchain('rustup','stable',env,ROOT)
        self.assertEqual(attempt.call_count,2)
        self.assertEqual(attempt.call_args_list[1].args[1]['RUSTUP_USE_CURL'],'1')
        self.assertEqual(env,{'PATH':'keep'})
        for call in attempt.call_args_list:
            self.assertNotIn('RUSTUP_DIST_SERVER',call.args[1])
            self.assertNotIn('CURL_SSL_NO_REVOKE',call.args[1])

    def test_existing_curl_configuration_can_fall_back_to_default(self):
        with patch.object(download,'_attempt',side_effect=[(1,'could not download file'),(0,'')]) as attempt:
            download.install_toolchain('rustup','stable',{'rustup_use_curl':'1'},ROOT)
        self.assertNotIn('rustup_use_curl',attempt.call_args.args[1])
        self.assertNotIn('RUSTUP_USE_CURL',attempt.call_args.args[1])

    def test_permission_failure_does_not_retry_as_a_network_error(self):
        with patch.object(download,'_attempt',return_value=(1,'Access denied. (os error 5)')) as attempt:
            with self.assertRaisesRegex(ValueError,'before compilation'):
                download.install_toolchain('rustup','stable',{},ROOT)
        self.assertEqual(attempt.call_count,1)

    def test_two_network_failures_stop_with_endpoint_and_no_endless_retry(self):
        with patch.object(download,'_attempt',return_value=(1,'error sending request: os error 10054')) as attempt:
            with self.assertRaisesRegex(ValueError,'https://static.rust-lang.org'):
                download.install_toolchain('rustup','stable',{},ROOT)
        self.assertEqual(attempt.call_count,2)

    def test_alternate_backend_unavailable_returns_real_diagnostic(self):
        with patch.object(download,'_attempt',side_effect=[(1,'could not download file'),(1,'curl backend unavailable')]):
            with self.assertRaisesRegex(ValueError,'curl backend unavailable'):
                download.install_toolchain('rustup','stable',{},ROOT)

if __name__=='__main__':unittest.main()
