"""Source acceptance checks usable without a game build; behavioral C test is separate."""
from pathlib import Path
import re
import unittest

SOURCE = Path(__file__).parents[1] / "platform" / "gw_profiler.c"

class ProfilerContract(unittest.TestCase):
    def test_disabled_paths_return_before_clock(self):
        source = SOURCE.read_text()
        for name in ("gw_prof_begin", "gw_prof_end", "gw_prof_counter", "gw_prof_sample",
                     "gw_prof_frame_begin", "gw_prof_frame_end", "gw_prof_cpu_completed",
                     "gw_prof_counter_detail", "gw_prof_gpu_completed", "gw_prof_try_cpu_completed"):
            body = source.split("void " + name + "(", 1)[1].split("\n}", 1)[0]
            self.assertRegex(body, r"if \(!gw_prof_enabled\(\)\) return;")
            before = body.split("if (!gw_prof_enabled()) return;", 1)[0]
            self.assertNotRegex(before, r"prof_now|malloc|calloc|realloc|prof_lock")

    def test_no_simulation_memory_or_rng_dependencies(self):
        source = SOURCE.read_text()
        self.assertNotRegex(source, r"gw_[rw](?:8|16|32|64|f32|ptr)\(|gw_mem1|gw_Snap|rand\(")

    def test_bounded_storage_and_schema(self):
        source = SOURCE.read_text()
        for token in ("PROF_EVENTS", "PROF_SAMPLES", "PROF_DEPTH", "schema_version",
                      "sample_window", "stack_overflow", "event_overwrites", "p99"):
            self.assertIn(token, source)

if __name__ == "__main__":
    unittest.main()
