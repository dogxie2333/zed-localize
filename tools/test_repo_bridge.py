"""Real Git/Dulwich checkout checks, including dirty-source rejection."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from repo_bridge import clean

class CheckoutTests(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory(prefix='zed-checkout-test-')
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)
        self.git = shutil.which('git')
        self.env = {key:value for key,value in os.environ.items() if not key.startswith('GIT_')}
        self.run_git('init','-q')
        for key,value in [('user.name','CI Fixture'),('user.email','ci-fixture@localhost'),
                          ('core.autocrlf','false'),('core.filemode','false'),('core.symlinks','false')]:
            self.run_git('config',key,value)
        for name in ('included','omitted'): (self.root/name).mkdir()
        (self.root/'.gitattributes').write_text('*.rs text eol=crlf\n',encoding='utf-8')
        (self.root/'included/public.rs').write_text('pub fn menu() {}\n',encoding='utf-8',newline='\r\n')
        (self.root/'omitted/other.rs').write_text('pub fn other() {}\n',encoding='utf-8',newline='\r\n')
        self.run_git('add','.')
        self.run_git('commit','-qm','Public test fixture')

    def run_git(self,*args):
        return subprocess.check_output([self.git,'-C',str(self.root),*args],env=self.env)

    def test_sparse_checkout_and_real_edits(self):
        self.run_git('sparse-checkout','init','--cone','--sparse-index')
        self.run_git('sparse-checkout','set','included')
        self.assertFalse((self.root/'omitted/other.rs').exists())
        self.assertTrue(clean(self.root))
        (self.root/'included/public.rs').write_text('pub fn PRIVATE_EDIT() {}\n',encoding='utf-8')
        self.assertFalse(clean(self.root))
        self.run_git('add','included/public.rs')
        self.assertFalse(clean(self.root))

    def test_line_endings_and_git_free_fallback(self):
        with patch('repo_bridge.shutil.which',return_value=None):
            self.assertTrue(clean(self.root))
        (self.root/'included/public.rs').write_bytes(b'pub fn menu() {}\r\n')
        self.assertTrue(clean(self.root))
        self.assertFalse(clean(self.root/'included'))
        (self.root/'included/public.rs').write_text('changed\n',encoding='utf-8')
        with patch('repo_bridge.shutil.which',return_value=None):
            self.assertFalse(clean(self.root))

    def test_windows_symlink_stub_changes_are_not_ignored(self):
        target=b'included/public.rs'
        blob=subprocess.check_output([self.git,'-C',str(self.root),'hash-object','-w','--stdin'],input=target,env=self.env).decode().strip()
        (self.root/'stub.txt').write_bytes(target)
        self.run_git('update-index','--add','--cacheinfo',f'120000,{blob},stub.txt')
        self.run_git('commit','-qm','Windows symlink representation')
        self.assertTrue(clean(self.root))
        (self.root/'stub.txt').write_bytes(b'omitted/other.rs')
        self.assertFalse(clean(self.root))

if __name__ == '__main__': unittest.main()
