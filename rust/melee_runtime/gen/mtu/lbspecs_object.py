"""object.c spec (see lbspecs.py)."""
from lbspecs import add

add("object", "src/sysdolphin/baselib/object.c", dict(pub_only=True, ncases=50),
    pub_only=True, own_hash=["hsdObj"])
