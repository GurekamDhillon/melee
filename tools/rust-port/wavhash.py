import sys
b=open(sys.argv[1],'rb').read()[44:]
keep=int(sys.argv[2]) if len(sys.argv)>2 else None
if keep: b=b[:keep*640]
h=0xcbf29ce484222325
for x in b: h=((h^x)*0x100000001b3)&0xFFFFFFFFFFFFFFFF
print(len(b)//640, hex(h))
