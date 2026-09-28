"""Host-only checks; never starts Melee."""
import unittest
import tempfile
from pathlib import Path
import sweep_all as sweep


class RunnerTests(unittest.TestCase):
    def test_incomplete_or_duplicate_sections_cannot_pass(self):
        log = 'TEST sweep-all section 1: PASS entered\n' * 3
        self.assertEqual(sweep.judge(log, '', 0, False, 3)['result'], 'INCOMPLETE')

    def test_crash_overrides_pass_and_reports_fault_not_stack_rva(self):
        log = '\n'.join(f'TEST sweep-all section {n}: PASS ok' for n in range(1, 4))
        log += '\ngw: FATAL ACCESS_VIOLATION at 123 melee-pc.map rva 0x00123456\n'
        log += 'gw: frames: melee-pc.map rva 0x00abcdef'
        row = sweep.judge(log, '', 0, False, 3)
        self.assertEqual((row['result'], row['rva']), ('CRASH', '0x00123456'))

    def test_clean_exit_requires_all_checks(self):
        log = '\n'.join(f'TEST sweep-all section {n}: PASS ok' for n in range(1, 4))
        self.assertEqual(sweep.judge(log, '', 0, False, 3)['result'], 'PASS')
        self.assertEqual(sweep.judge(log, '', 1, False, 3)['result'], 'EXIT')
        self.assertEqual(sweep.judge(log, '', -1, True, 3)['result'], 'TIMEOUT')

    def test_no_stack_address_mislabeled_as_fault(self):
        row = sweep.judge('FATAL driver.dll+0x123\nframes: melee-pc.map rva 0x1234', '', 1, False, 3)
        self.assertIsNone(row['rva'])

    def test_missing_discovery_fails_closed(self):
        with self.assertRaises(ValueError):
            sweep.catalog('stage tables ready: 221 internal, 328 external')

    def test_plan_covers_real_steps_and_every_discovered_kind(self):
        cat = sweep.catalog('94 m-ex fighter kinds from MxDt.dat (kinds 33..126)\n'
                            'stage tables ready: 221 internal, 328 external')
        cases = sweep.plan(cat)
        self.assertEqual(sum(c['group'] == 'classic' for c in cases), 11)
        self.assertEqual(sum(c['group'] == 'adventure' for c in cases), 12)
        fighters = [c for c in cases if c['group'] == 'fighter']
        self.assertEqual([c['fighter_kind'] for c in fighters], list(range(33, 127)))
        self.assertIn('stage=ext:327', '\n'.join(c['scene'] for c in cases))
        self.assertEqual(len(sweep.subset()), 6)

    def test_known_issues_match_exact_signature(self):
        row = dict(tag='vs-ext2', result='CRASH', rva='0x00123456', detail='FATAL x')
        issue = dict(row, reason='confirmed defect')
        self.assertTrue(sweep.is_known(row, [issue]))
        self.assertFalse(sweep.is_known(dict(row, rva='0x00123457'), [issue]))
        self.assertFalse(sweep.is_known(dict(row, result='TIMEOUT'), [issue]))

    def test_known_fault_survives_aslr(self):
        issue = dict(tag='fighter-fk33', result='CRASH', rva='0x00123456',
                     detail='gw: FATAL AV at 00456789  melee-pc.map rva 0x00123456')
        row = dict(issue, detail='gw: FATAL AV at 006789AB  melee-pc.map rva 0x00123456')
        self.assertTrue(sweep.is_known(row, [issue]))

    def test_summary_lists_only_new_failures_but_json_keeps_every_case(self):
        rows = [dict(tag='good', result='PASS', detail='3 checks passed', rva=None, tail='', scene='x'),
                dict(tag='known', result='CRASH', detail='FATAL AV', rva='0x00001234', tail='old', scene='y'),
                dict(tag='new', result='TIMEOUT', detail='deadline', rva=None, tail='last heartbeat', scene='z')]
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self.assertTrue(sweep.report(root, rows, [rows[1]], {}))
            summary = (root / 'summary.md').read_text(encoding='utf-8')
            self.assertNotIn('## known', summary)
            self.assertNotIn('## good', summary)
            self.assertIn('## new', summary)
            self.assertIn('last heartbeat', summary)
            self.assertIn('unavailable', summary)


if __name__ == '__main__':
    unittest.main()
