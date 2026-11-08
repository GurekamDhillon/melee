"""What Hero's sandboxed mapper (map_game_image, used inside an AppContainer) would make of a core v1 game image:
base, size against the held range (0x82800000..0x84000000), imports, implicit TLS and relocations.

    python pe_check.py <melee_game.dll>
"""
import struct
import sys

BASE, HOLD_END = 0x82800000, 0x84000000


def main():
    d = open(sys.argv[1], "rb").read()
    pe = struct.unpack_from("<I", d, 0x3C)[0]
    nsec, = struct.unpack_from("<H", d, pe + 6)
    opt = pe + 24
    image_base, = struct.unpack_from("<Q", d, opt + 24)
    size_image, = struct.unpack_from("<I", d, opt + 56)
    dll_chars, = struct.unpack_from("<H", d, opt + 70)
    dirs = opt + 112
    ddir = [struct.unpack_from("<II", d, dirs + 8 * i) for i in range(16)]
    secs = []
    for i in range(nsec):
        s = opt + struct.unpack_from("<H", d, pe + 20)[0] + 40 * i
        name = d[s:s + 8].rstrip(b"\0").decode()
        vsize, va, rsize, rptr = struct.unpack_from("<IIII", d, s + 8)
        secs.append((name, va, vsize, rptr, rsize))

    def off(rva):
        for _, va, vs, rp, rs in secs:
            if va <= rva < va + max(vs, rs):
                return rva - va + rp
        raise ValueError(hex(rva))

    def cstr(o):
        return d[o:d.index(b"\0", o)].decode()

    print(f"ImageBase {image_base:#x} ({'ok' if image_base == BASE else 'NOT ' + hex(BASE)})")
    room = HOLD_END - BASE
    print(f"SizeOfImage {size_image:#x} ({size_image / 2**20:.1f} MB); the mapper's limit {room:#x} ({room / 2**20:.0f} MB): "
          f"{'fits' if size_image <= room else 'REFUSED (its address range is not held)'}")
    print(f"DYNAMICBASE {'on' if dll_chars & 0x40 else 'off'}; relocation table {ddir[5][1]} bytes "
          f"(the mapper applies none: the image must sit at its base)")
    for name, va, vs, rp, rs in secs:
        print(f"  section {name:8} rva {va:#9x} vsize {vs:#9x}")
    imp_rva, imp_size = ddir[1]
    dlls = []
    o = off(imp_rva)
    while True:
        oft, _, _, name_rva, ft = struct.unpack_from("<IIIII", d, o)
        if not name_rva:
            break
        dlls.append(cstr(off(name_rva)))
        o += 20
    print(f"imports from {len(dlls)} DLLs: {', '.join(dlls)}")
    tls_rva, tls_size = ddir[9]
    if tls_size:
        start, end, index_va, cb_va, zero_fill, _ = struct.unpack_from("<QQQQII", d, off(tls_rva))
        print(f"implicit TLS: template {end - start} bytes + {zero_fill} zero-fill, index slot at {index_va:#x}: "
              f"{'USED (the mapper sets up no TLS slot: thread_local! accesses would read slot 0 of another module)' if end > start or zero_fill else 'empty'}")
    else:
        print("implicit TLS: none")


if __name__ == "__main__":
    main()
