"""Validate the standalone C fixture's real JSON, after the integrator executes it.

Usage from the fixture output directory: python <repo>/melee/pc/tests/validate_profiler_fixture.py
Not a mocked/model output test: absence of executable output fails loudly.
"""
import json
from pathlib import Path

report = json.loads(Path("profiler-test-report.json").read_text())
assert report["schema_version"] == 1
assert report["percentile_scope"] == "deterministic_run_reservoir"
assert report["gpu"]["available"] is True
gpu = report["zones"]["gpu"]
assert gpu == dict(count=100, mean=50.5, p50=50, p95=95, p99=99, max=100)
assert report["diagnostics"]["event_overwrites"] == 0
assert report["detail_names"]["99"] == 'gpu pass test'
assert report["detail_names"]["100"] == 'test "gpu"\npass'
# The fixture fills the identity table before restoring GPU samples. The
# reserved mixed bucket retains all samples even when detail 99 cannot fit.
gpu_detail = next(d for d in report["details"] if d["zone"] == "gpu" and d["detail"] == 0xFFFFFFFF)
assert gpu_detail == dict(zone="gpu", detail=0xFFFFFFFF, **gpu)
trace = json.loads(Path("profiler-test-trace.json").read_text())
assert trace["event_capacity"] == 131072
assert len(trace["traceEvents"]) == 9
assert report["diagnostics"]["event_sampled"] == 131071
assert report["diagnostics"]["detail_aggregated"] > 0
cause = trace["traceEvents"][-1]["args"]
assert cause["last_script_call"] == "stage_add_line"
assert cause["last_spawn"] == "Goomba"
assert cause["last_area"] == "maze-room-7"
gpu_trace = json.loads(Path("profiler-test-gpu.json").read_text())
event = next(e for e in gpu_trace["traceEvents"] if e["name"] == "gpu_pass/gpu pass test")
assert event["ph"] == "C" and event["args"]["render_frame"] == 42
assert event["args"]["duration_ms"] == 2.5
assert event["args"]["timestamp_source"] == "callback_arrival"
assert event["args"]["detail_name"] == "gpu pass test"
frames = json.loads(Path("profiler-test-frames.json").read_text())
assert (frames["first_frame"], frames["last_frame"]) == (3, 4)
assert all(e["args"]["frame"] in (3, 4) for e in frames["traceEvents"])
hitch = json.loads(Path("profiler-trace.json.hitch-1.json").read_text())
assert hitch["last_frame"] == 3
assert any(e["args"].get("frame") == 1 for e in hitch["traceEvents"])
print("profiler fixture schema/percentiles/trace/hitch: PASS")
