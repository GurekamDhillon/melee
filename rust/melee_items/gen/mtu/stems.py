"""Per-TU driver descriptions for mk_driver.py (see there)."""
STEMS = {}
IDX4 = ("fv() & 3", "(fv() & 3) as i32")
STEMS["ithitbox"] = dict(
    ov={"it_80275594": {1: IDX4}, "it_8027570C": {1: IDX4}, "it_8027572C": {1: IDX4}, "it_80275820": {3: IDX4}},
)
