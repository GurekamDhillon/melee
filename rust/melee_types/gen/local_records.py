"""Extract TU-local records into the single melee_types layout probe.

Bodies come from the C source, never a separately maintained Rust definition.
The one native widening below preserves a callback pointer which C stored in u32;
uintptr_t is still four bytes in the PPC parse, so the GC metadata is unchanged.
"""
import re


def record(text, pattern):
    match = re.search(pattern, text)
    if not match:
        raise ValueError(f"record not found: {pattern}")
    start = match.start()
    brace = text.index("{", match.start())
    depth = 1
    end = brace + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    end = text.index(";", end) + 1
    return text[start:end]


def source_records(root):
    parts = ["/* TU-local records extracted by gen/local_records.py. */"]
    for stem, names in [
        ("lbmemory", ["MemEntry", "LBMgr", "Allocator"]),
        ("lbheap", ["lbHeap_HeapDesc", "lbHeap_HeapOffsetView"]),
    ]:
        source = (root / f"src/melee/lb/{stem}.c").read_text()
        for name in names:
            body = record(source, rf"struct {name}\s*\{{")
            if name == "LBMgr":
                body = body.replace("u32 cb_arg;", "uintptr_t cb_arg;")
            parts.append(body)
    source = (root / "src/melee/lb/lbmthp.c").read_text()
    for name in ["THPFrameBuffer", "THPDecComp"]:
        # Each is a named typedef of an anonymous struct.
        for match in re.finditer(r"typedef struct(?:\s+\w+)?\s*\{", source):
            body = record(source[match.start():], r"typedef struct(?:\s+\w+)?\s*\{")
            if re.search(rf"\}}\s*{name};$", body):
                parts.append(body)
                break
        else:
            raise ValueError(name)
    source = (root / "src/melee/lb/lbrefract.c").read_text()
    for variable, name in [("lbl_804336D0", "lbRefract_State"), ("refract_data", "lbRefract_Data")]:
        for match in re.finditer(r"static struct\s*\{", source):
            body = record(source[match.start():], r"static struct\s*\{")
            if re.search(rf"\}}\s*\*?\s*{variable};$", body):
                body = body.replace("static struct", f"struct {name}", 1)
                body = re.sub(rf"\}}\s*\*?\s*{variable};$", "};", body)
                parts.append(body)
                break
        else:
            raise ValueError(variable)
    parts.append(record(source, r"struct lbRefract_DataLayout\s*\{"))
    return "\n".join(parts) + "\n"
