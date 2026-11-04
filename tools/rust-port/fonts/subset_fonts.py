"""Subset the UI fonts (Inter, JetBrains Mono; SIL OFL 1.1) for rust/melee_gx/assets/fonts.

Keeps ASCII, Latin-1 and the kit's symbols, drops hinting/GSUB/GPOS, and bakes GPOS pair kerning for the kept
characters into a legacy `kern` table (format 0) that ab_glyph reads. Usage:
  python -I subset_fonts.py <Inter-Regular.ttf> <Inter-Bold.ttf> <JBM-Regular.ttf> <JBM-Bold.ttf> <out_dir>
"""
import sys, os
from fontTools.ttLib import TTFont, newTable
from fontTools.ttLib.tables._k_e_r_n import KernTable_format_0
from fontTools import subset

CHARS = list(range(0x20, 0x7F)) + list(range(0xA0, 0x100)) + [0x25C0, 0x25B6, 0x203A, 0x25CF, 0x258D, 0x25C6, 0x2026]
KERN_CHARS = list(range(0x20, 0x7F))


def gpos_pairs(font, glyphs):
    """(left, right) -> x advance adjustment, from GPOS PairPos lookups (first hit wins, like a shaper)."""
    out = {}
    if 'GPOS' not in font:
        return out
    gs = set(glyphs)
    for lk in font['GPOS'].table.LookupList.Lookup:
        subs = lk.SubTable
        if lk.LookupType == 9:
            subs = [s.ExtSubTable for s in subs]
        for st in subs:
            if getattr(st, 'LookupType', 2) != 2 and lk.LookupType != 2 and lk.LookupType != 9:
                continue
            if not hasattr(st, 'Format') or not hasattr(st, 'Coverage'):
                continue
            cov = st.Coverage.glyphs
            if st.Format == 1:
                for i, g1 in enumerate(cov):
                    if g1 not in gs:
                        continue
                    for pvr in st.PairSet[i].PairValueRecord:
                        g2 = pvr.SecondGlyph
                        v = getattr(pvr.Value1, 'XAdvance', 0) if pvr.Value1 else 0
                        if g2 in gs and v and (g1, g2) not in out:
                            out[(g1, g2)] = v
            elif st.Format == 2:
                c1 = st.ClassDef1.classDefs
                c2 = st.ClassDef2.classDefs
                for g1 in cov:
                    if g1 not in gs:
                        continue
                    r1 = st.Class1Record[c1.get(g1, 0)]
                    for g2 in glyphs:
                        rec = r1.Class2Record[c2.get(g2, 0)]
                        v = getattr(rec.Value1, 'XAdvance', 0) if rec.Value1 else 0
                        if v and (g1, g2) not in out:
                            out[(g1, g2)] = v
    return out


def main():
    *ins, out_dir = sys.argv[1:]
    os.makedirs(out_dir, exist_ok=True)
    for path in ins:
        f = TTFont(path)
        cmap = f.getBestCmap()
        kg = [cmap[c] for c in KERN_CHARS if c in cmap]
        pairs = gpos_pairs(f, kg)
        opts = subset.Options()
        opts.hinting = False
        opts.layout_features = []
        opts.drop_tables += ['GSUB', 'GPOS', 'GDEF', 'STAT', 'DSIG', 'meta']
        opts.name_IDs = ['*']
        opts.notdef_outline = True
        s = subset.Subsetter(opts)
        s.populate(unicodes=CHARS)
        s.subset(f)
        if pairs:
            k = newTable('kern')
            k.version = 0
            t = KernTable_format_0()
            t.version, t.format, t.coverage, t.tupleIndex = 0, 0, 1, None
            t.kernTable = pairs
            k.kernTables = [t]
            f['kern'] = k
        name = os.path.basename(path)
        f.save(os.path.join(out_dir, name))
        print(name, len(pairs), 'kern pairs', os.path.getsize(os.path.join(out_dir, name)), 'bytes')


main()
