"""Source-maths audit, without a build or GPU execution.

Evaluates the production presentation-angle expression against independently
intersected perspective edge rays. Replacing atan(tan(fov/2)*aspect) with the
retail fov*aspect approximation must fail. This does not diagnose floor draws.
"""
import math
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[3]


def expression(text):
    text = re.sub(r"(?<=\d)f\b", "", text)
    for old, new in {
        "HSD_CObjGetFov(cobj)": "fov",
        "HSD_CObjGetAspect(cobj)": "aspect",
        "MTXDegToRad": "radians",
        "atanf": "atan", "tanf": "tan",
    }.items():
        text = text.replace(old, new)
    return compile(text.strip(), "<production angle>", "eval")


def production_angle():
    source = (ROOT / "melee/src/melee/cm/camera.c").read_text()
    bounds = source.split("static bool Camera_Bounds(", 1)[1]
    angle = re.search(r"\} else \{\s*half_fov = (.*?);", bounds, re.S)
    if angle is None:
        raise AssertionError("Could not extract presentation bounds expression")
    return expression(angle.group(1))


def presentation_source():
    source = (ROOT / "melee/src/melee/cm/camera.c").read_text().split("static bool Camera_Bounds(", 1)[1].split("return result;", 1)[0]
    # Select actual TARGET_PC assignments, excluding the retail #else copies.
    return re.sub(r"#if defined\(TARGET_PC\)\s*(.*?)#else\s*.*?#endif", r"\1", source, flags=re.S)


def evaluate(code, fov, aspect):
    return eval(code, {"__builtins__": {}}, {
        "fov": fov, "aspect": aspect, "radians": math.radians,
        "atan": math.atan, "tan": math.tan,
    })


def projected_bounds(angle, yaw, eye_x, eye_z, source=None):
    # Camera_Bounds' rotation/intersection with z=0; an edge pointing past
    # the plane's horizon cannot impose a finite culling bound.
    forward_x, forward_z = math.sin(yaw), math.cos(yaw)
    if abs(forward_x) <= 0.0001 or abs(forward_z) <= 0.0001:
        # Actual Camera_Bounds fallback for an axis-aligned camera.
        return -8.5070587e37, 8.5070587e37
    result = []
    for index, side in enumerate((angle, -angle)):
        dx = forward_x * math.cos(side) + forward_z * math.sin(side)
        dz = forward_z * math.cos(side) - forward_x * math.sin(side)
        if abs(dz) > 0.0001 and forward_z * dz > 0:
            result.append(eye_x - eye_z * dx / dz)
        else:
            if source is None:
                source = presentation_source()
            field = "left" if index == 0 else "right"
            block = source.split(f"else if (edge_x{'2' if index else ''} > 0.0f)", 1)[1]
            assignments = re.findall(rf"\*{field} = ([^;]+);", block)
            raw = assignments[0 if dx > 0 else 1]
            # TARGET_PC presentation branch of a logic/presentation ternary.
            raw = raw.split(":", 1)[-1] if "?" in raw else raw
            result.append(float(raw.strip().rstrip("f")))
    return min(result), max(result)


class FrustumAudit(unittest.TestCase):
    def test_old_horizon_sentinel_excludes_visible_ray(self):
        # Regression control, without reverting a concurrently edited file.
        source = presentation_source()
        source = source.replace("logic ? 8.5070587e37f : -8.5070587e37f", "8.5070587e37f")
        source = source.replace("logic ? -8.5070587e37f : 8.5070587e37f", "-8.5070587e37f")
        angle = evaluate(production_angle(), 60, 4 / 3)
        _, high = projected_bounds(angle, math.radians(-65), 17, 80, source)
        self.assertLess(high, 1176)  # Real visible ray is x=1176.089479.

    def test_dense_visible_edges_fit_presentation_bounds(self):
        code = production_angle()
        source = presentation_source()
        checks = 0
        for step in range(2601):
            requested = 1.0 + step / 1000.0
            # The existing supported policy fits bars outside 4:3..32:9.
            aspect = min(max(requested, 4 / 3), 32 / 9)
            for fov in (15, 30, 60, 90):
                slope = math.tan(math.radians(fov / 2)) * aspect
                angle = evaluate(code, fov, aspect)
                for degrees in (-65, -30, -1, 0, 1, 30, 65):
                    yaw = math.radians(degrees)
                    low, high = projected_bounds(angle, yaw, 17, 80, source)
                    for ndc_x in (-1, -0.5, 0, 0.5, 1):
                        # Independently unproject NDC x and rotate the ray.
                        dx = math.sin(yaw) + ndc_x * slope * math.cos(yaw)
                        dz = math.cos(yaw) - ndc_x * slope * math.sin(yaw)
                        if dz > 0.0001:
                            visible_x = 17 - 80 * dx / dz
                            self.assertLessEqual(low - 1e-6, visible_x)
                            self.assertGreaterEqual(high + 1e-6, visible_x)
                            checks += 1
                    if degrees and abs(degrees) + math.degrees(angle) < 89:
                        # A finite bound must also be tight, so reverting to
                        # the retail approximation cannot pass by overculling.
                        x0 = 17 - 80 * (math.sin(yaw) - slope * math.cos(yaw)) / (math.cos(yaw) + slope * math.sin(yaw))
                        x1 = 17 - 80 * (math.sin(yaw) + slope * math.cos(yaw)) / (math.cos(yaw) - slope * math.sin(yaw))
                        self.assertAlmostEqual(low, min(x0, x1), places=6)
                        self.assertAlmostEqual(high, max(x0, x1), places=6)
        print(f"frustum audit: 2601 requested aspects, {checks} visible rays")

    def test_catches_retail_angle_approximation(self):
        wrong = expression("0.5f * MTXDegToRad(HSD_CObjGetFov(cobj)) * HSD_CObjGetAspect(cobj)")
        # The retail approximation is not the tight perspective edge.
        aspect = 73 / 60
        angle = evaluate(wrong, 30, aspect)
        yaw = math.radians(1)
        _, high = projected_bounds(angle, yaw, 0, 80)
        slope = math.tan(math.radians(15)) * aspect
        visible_edge = -80 * (math.sin(yaw) - slope * math.cos(yaw)) / (math.cos(yaw) + slope * math.sin(yaw))
        self.assertGreater(abs(high - visible_edge), 0.1)


if __name__ == "__main__":
    unittest.main()
