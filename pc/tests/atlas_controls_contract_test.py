"""The picker exposes online Controls by default; loading keeps its opt-in.

The workspace's older select-enum test still pins both gates to zero. This
worktree-owned contract captures the intentional split without changing it.
"""
import re
import unittest
from pathlib import Path


class OnlineControlsGate(unittest.TestCase):
    def test_picker_and_loading_defaults(self):
        game = Path(__file__).resolve().parents[2]
        text = (game / 'src/melee/gm/gmfrontend_atlas_select.inc').read_text()
        picker = text.split('static int fas_open_css(void)', 1)[1].split('static ', 1)[0]
        loading = text.split('static int fas_open_loading(void)', 1)[1].split('static ', 1)[0]
        self.assertRegex(picker, r'Settings_Int\("atlas_lobby",\s*1\)\s*==\s*0')
        self.assertRegex(loading, r'Settings_Int\("atlas_lobby",\s*0\)\s*==\s*0')
        self.assertIn('Z:Controls', text)
        self.assertRegex(text, r'FA_SELEV_CONTROLS\)\s*\{\s*ev\s*\|=\s*Ui_CssControl')


if __name__ == '__main__':
    unittest.main()
