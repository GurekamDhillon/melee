#!/usr/bin/env python3
"""Drive a running melee-pc.exe through its console socket (MELEE_CONSOLE_PORT).

    python pc/scripts/console.py 51700 "state" "savestate 1" "step 30" "= gd.player(1).x"
    python pc/scripts/console.py 51700            # interactive: one command per line

Each command's output is printed; the exit status is 1 if any command answered ">>> error".
The protocol is plain text (docs/scripting.md, "The console socket"): send one line, read lines
until ">>> ok" or ">>> error".
"""
import socket
import sys


def run(sock_file, sock, line):
    sock.sendall((line + "\n").encode("utf-8"))
    out = []
    while True:
        reply = sock_file.readline()
        if not reply:
            return out, False
        reply = reply.rstrip("\n")
        if reply in (">>> ok", ">>> error"):
            return out, reply == ">>> ok"
        out.append(reply)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    port = int(sys.argv[1])
    sock = socket.create_connection(("127.0.0.1", port), timeout=30)
    f = sock.makefile("r", encoding="utf-8", errors="replace")
    f.readline()  # banner
    cmds = sys.argv[2:]
    ok_all = True
    if cmds:
        for c in cmds:
            out, ok = run(f, sock, c)
            print("> " + c)
            for o in out:
                if not o.startswith("> "):
                    print(o)
            ok_all &= ok
    else:
        for line in sys.stdin:
            out, ok = run(f, sock, line.rstrip("\n"))
            for o in out:
                if not o.startswith("> "):
                    print(o)
            ok_all &= ok
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
