#!/usr/bin/env python3
"""scenarios.json -> scenarios.tsv (name, kind, frames, pad, scene), the table `melee golden --engine tu` reads for the coverage scenarios."""
import json, os
root = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', '..', 'docs', 'rust-port', 'golden', 'coverage')
s = json.load(open(os.path.join(root, 'scenarios.json')))['scenarios']
with open(os.path.join(root, 'scenarios.tsv'), 'w', newline='\n') as f:
    f.write("# name\tkind\tframes\tpad\tscene   (generated from scenarios.json by tools/rust-port/golden/coverage/gen_tsv.py; read by melee golden --engine tu)\n")
    for k, v in s.items():
        f.write(f"{k}\t{v['kind']}\t{v['frames']}\t{os.path.basename(v['pad'])}\t{v['scene']}\n")
