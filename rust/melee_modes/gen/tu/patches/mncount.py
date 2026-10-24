"""Oracle patch for src/melee/mn/mncount.c (applied to a copy; see gen/tu/run.py).

`mnCount_CreateRow` passes `buf` and `(char*) buf - 4` to the text formatter: on PPC the two 4-byte
stack buffers are adjacent. The native C would read before a local array (undefined), so the copy
makes `buf` the upper half of one 8-byte zeroed area; the Rust port does the same.
"""


def patch(t):
    a = "char buf[4];"
    assert t.count(a) == 1
    return t.replace(a, "char buf_[8] = { 0 }; char* buf = buf_ + 4;")
