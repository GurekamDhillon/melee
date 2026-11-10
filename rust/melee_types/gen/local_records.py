"""Extract TU-local records into the single melee_types layout probe.

Bodies come from the C source, never a separately maintained Rust definition.
"""
import re


def record(text, pattern):
    match = re.search(pattern, text, re.M)
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
        # upstream #3547: MemEntry became lbmemory.h's HSD_AllocEntry, LBMgr.cb_arg is uintptr_t in the C itself, the
        # heap offset view is gone and `struct Heap` / lbHeap_HeapState moved from lbheap.static.h into lbheap.c
        ("lbmemory", ["LBMgr", "Allocator"]),
        ("lbheap", ["Heap", "lbHeap_HeapState", "lbHeap_HeapDesc"]),
    ]:
        source = (root / f"src/melee/lb/{stem}.c").read_text()
        for name in names:
            body = record(source, rf"^struct {name}\s*\{{")
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
    # upstream #3583 named the records: `static struct lbl_804336D0_t {...} lbl_804336D0;` in lbrefract.c and
    # `struct refract_data_t` in lbrefract.h (refract_data is a pointer to it). The port keeps the generated names.
    for match in re.finditer(r"static struct(?:\s+\w+)?\s*\{", source):
        body = record(source[match.start():], r"static struct(?:\s+\w+)?\s*\{")
        if re.search(r"\}\s*\*?\s*lbl_804336D0;$", body):
            body = re.sub(r"^static struct(?:\s+\w+)?\s*\{", "struct lbRefract_State {", body, count=1)
            body = re.sub(r"\}\s*\*?\s*lbl_804336D0;$", "};", body)
            parts.append(body)
            break
    else:
        raise ValueError("lbl_804336D0")
    header = (root / "src/melee/lb/lbrefract.h").read_text()
    body = record(header, r"struct refract_data_t\s*\{")
    parts.append(body.replace("struct refract_data_t", "struct lbRefract_Data", 1))
    parts.append(record(source, r"struct lbRefract_DataLayout\s*\{"))
    return "\n".join(parts) + "\n"
