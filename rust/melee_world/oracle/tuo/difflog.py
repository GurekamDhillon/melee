import sys
a = open(sys.argv[1], errors="replace").read().splitlines()
b = open(sys.argv[2], errors="replace").read().splitlines()
print("C lines", len(a), "Rust lines", len(b))
n = 0
for i in range(max(len(a), len(b))):
    x = a[i] if i < len(a) else "<none>"
    y = b[i] if i < len(b) else "<none>"
    if x != y:
        k = next((j for j in range(min(len(x), len(y))) if x[j] != y[j]), min(len(x), len(y)))
        lo = max(0, k - 60)
        print(f"line {i}: first diff at col {k}")
        print("  C : ..." + x[lo:k + 60])
        print("  Rs: ..." + y[lo:k + 60])
        n += 1
        if n >= 6:
            break
