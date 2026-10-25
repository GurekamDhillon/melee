"""Lead-only Windows smoke driver: CSS, HUD and results client-window PNGs by PID.
Start an owned run.sh sandbox at VS/CSS first; this script never starts the game.
Requires Pillow and the workspace tools on PYTHONPATH. All captures stay in the
sandbox, outside the game repository. No disc fixture is read or generated here.
"""
import argparse
import ctypes as C
from ctypes import wintypes as W
import json
from pathlib import Path
import time

from PIL import Image
from tools.port import runs
from tools.skins.skin_shot import Console


def _print_window(user, hwnd, width, height):
    """Client-area capture through PrintWindow; no focus change, no screen grab."""
    gdi = C.WinDLL("gdi32", use_last_error=True)
    user.GetDC.argtypes = [W.HWND]
    user.GetDC.restype = W.HDC
    user.ReleaseDC.argtypes = [W.HWND, W.HDC]
    user.PrintWindow.argtypes = [W.HWND, W.HDC, W.UINT]
    gdi.CreateCompatibleDC.argtypes = [W.HDC]
    gdi.CreateCompatibleDC.restype = W.HDC
    gdi.CreateCompatibleBitmap.argtypes = [W.HDC, C.c_int, C.c_int]
    gdi.CreateCompatibleBitmap.restype = W.HBITMAP
    gdi.SelectObject.argtypes = [W.HDC, W.HGDIOBJ]
    gdi.SelectObject.restype = W.HGDIOBJ
    gdi.DeleteObject.argtypes = [W.HGDIOBJ]
    gdi.DeleteDC.argtypes = [W.HDC]

    class BMI(C.Structure):
        _fields_ = [("size", W.DWORD), ("w", W.LONG), ("h", W.LONG), ("planes", W.WORD),
                    ("bpp", W.WORD), ("comp", W.DWORD), ("img", W.DWORD), ("xp", W.LONG),
                    ("yp", W.LONG), ("used", W.DWORD), ("imp", W.DWORD)]

    gdi.GetDIBits.argtypes = [W.HDC, W.HBITMAP, W.UINT, W.UINT, C.c_void_p, C.c_void_p, W.UINT]
    screen = user.GetDC(None)
    mem = gdi.CreateCompatibleDC(screen)
    bmp = gdi.CreateCompatibleBitmap(screen, width, height)
    old = gdi.SelectObject(mem, bmp)
    try:
        # PW_CLIENTONLY | PW_RENDERFULLCONTENT
        if not user.PrintWindow(hwnd, mem, 1 | 2):
            raise RuntimeError("PrintWindow failed")
        info = BMI(C.sizeof(BMI), width, -height, 1, 32, 0, 0, 0, 0, 0, 0)
        buf = C.create_string_buffer(width * height * 4)
        if not gdi.GetDIBits(mem, bmp, 0, height, buf, C.byref(info), 0):
            raise RuntimeError("GetDIBits failed")
        return Image.frombuffer("RGBA", (width, height), buf.raw, "raw", "BGRA", 0, 1).convert("RGB")
    finally:
        gdi.SelectObject(mem, old)
        gdi.DeleteObject(bmp)
        gdi.DeleteDC(mem)
        user.ReleaseDC(None, screen)


def window_capture(record, sandbox, name):
    """Verify PID + creation time + executable, then capture only its client area."""
    api = runs.WinAPI()
    process = api.k.OpenProcess(0x1000, False, record["pid"])
    if not process:
        raise RuntimeError("owned game has exited")
    try:
        if (api.created(process) != record["process_created"] or
                runs.normalized(api.path(process)) != runs.normalized(sandbox / "melee-pc.exe")):
            raise RuntimeError("run.json no longer identifies this process")
        user = C.WinDLL("user32", use_last_error=True)
        user.SetProcessDPIAware()
        callback_type = C.WINFUNCTYPE(W.BOOL, W.HWND, W.LPARAM)
        user.EnumWindows.argtypes = [callback_type, W.LPARAM]
        user.GetWindowThreadProcessId.argtypes = [W.HWND, C.POINTER(W.DWORD)]
        user.IsWindowVisible.argtypes = [W.HWND]
        user.GetClientRect.argtypes = [W.HWND, C.POINTER(W.RECT)]
        user.ClientToScreen.argtypes = [W.HWND, C.POINTER(W.POINT)]
        windows = []

        @callback_type
        def visit(hwnd, _):
            pid = W.DWORD()
            user.GetWindowThreadProcessId(hwnd, C.byref(pid))
            rect = W.RECT()
            if (pid.value == record["pid"] and user.IsWindowVisible(hwnd) and
                    user.GetClientRect(hwnd, C.byref(rect)) and rect.right >= 320 and rect.bottom >= 240):
                windows.append(hwnd)
            return True

        user.EnumWindows(visit, 0)
        if len(windows) != 1:
            raise RuntimeError("expected one visible game window for owned PID")
        hwnd = windows[0]
        # Hands-off: never restore, activate or focus the window, and never grab the
        # screen (the user is at the keyboard). PrintWindow asks the window to paint
        # itself into an off-screen bitmap, so alt-tabbing cannot change the result.
        rect = W.RECT()
        if not user.GetClientRect(hwnd, C.byref(rect)):
            raise RuntimeError("cannot locate the owned client window")
        image = _print_window(user, hwnd, rect.right, rect.bottom)
        target = sandbox / (name + ".png")
        image.save(target)
        print(f"pid={record['pid']} window={int(hwnd)} {target}")
    finally:
        api.k.CloseHandle(process)




def verify_process(record, sandbox):
    api = runs.WinAPI()
    process = api.k.OpenProcess(0x1000, False, record["pid"])
    if not process:
        raise RuntimeError("owned game has exited")
    try:
        if (api.created(process) != record["process_created"] or
                runs.normalized(api.path(process)) != runs.normalized(sandbox / "melee-pc.exe")):
            raise RuntimeError("run.json no longer identifies this process")
    finally:
        api.k.CloseHandle(process)

def verify_console_owner(port, pid):
    # Do not send game-changing commands to another sandbox on a reused port.
    import subprocess
    query = ("Get-NetTCPConnection -State Listen -LocalPort " + str(int(port)) +
             " -ErrorAction Stop | Select-Object -ExpandProperty OwningProcess -Unique")
    result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", query],
                            capture_output=True, text=True, check=True,
                            creationflags=subprocess.CREATE_NO_WINDOW)
    owners = {int(line.strip()) for line in result.stdout.splitlines() if line.strip()}
    if owners != {pid}:
        raise RuntimeError("console port does not belong to the owned game PID")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--fighter", default="geno:geno-falcon")
    args = parser.parse_args()
    sandbox = args.run.resolve()
    deadline = time.monotonic() + 100
    while not (sandbox / "run.json").is_file():
        if time.monotonic() >= deadline:
            raise TimeoutError("no owned run metadata")
        time.sleep(0.1)
    record = runs.read_json(sandbox / "run.json")
    console = Console(args.port)
    if not console.connect(time.time() + 25):
        raise RuntimeError("no console for capture run")
    console.sock.settimeout(5)
    verify_console_owner(args.port, record["pid"])

    def command(text):
        verify_process(record, sandbox)
        output, ok = console.run(text)
        if not ok:
            raise RuntimeError("console refused: " + " ".join(output))
        return output

    def wait_for(expression):
        while time.monotonic() < deadline:
            if any(line.strip() == "true" for line in command("= " + expression)):
                return
            time.sleep(0.2)
        raise TimeoutError(expression)

    try:
        wait_for('gd.scene().name == "GS_CSS" or gd.menu().title == "CHARACTER SELECT"')
        time.sleep(2)
        window_capture(record, sandbox, "css")
        # A scripted normal stock-match finish exercises the actual results card.
        scene = (f"mode=vs;at=match;p1={args.fighter}/hu/c0;"
                 "p2=captain/cpu0/c0;stage=fd;match=stock;stocks=1;time=0;items=off")
        command("= gd.scene_launch(" + json.dumps(scene) + ")")
        wait_for("gd.match().active and gd.match().frame > 120 and gd.player(2) ~= nil")
        window_capture(record, sandbox, "stock")
        command('= gd.set_stocks(2, 1); gd.teleport(2, 0, -400)')
        wait_for('gd.scene().name == "GS_RESULTS"')
        time.sleep(4)
        window_capture(record, sandbox, "results")
        # Re-enter CSS to expose stale scene-archive pointers in the next screen.
        command("= gd.scene_launch(" + json.dumps(scene.replace("at=match", "at=css")) + ")")
        wait_for('gd.scene().name == "GS_CSS" or gd.menu().title == "CHARACTER SELECT"')
        time.sleep(2)
        window_capture(record, sandbox, "css-return")
    finally:
        try:
            verify_process(record, sandbox)
            console.run("quit")
        except (OSError, ConnectionError, RuntimeError):
            pass
        console.sock.close()


if __name__ == "__main__":
    main()
