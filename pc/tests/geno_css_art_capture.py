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

from PIL import ImageGrab
from tools.port import runs
from tools.skins.skin_shot import Console


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
        user.ShowWindow.argtypes = [W.HWND, C.c_int]
        user.SetForegroundWindow.argtypes = [W.HWND]
        user.GetForegroundWindow.restype = W.HWND
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
        user.ShowWindow(hwnd, 9)  # SW_RESTORE
        user.SetForegroundWindow(hwnd)
        time.sleep(0.5)
        if user.GetForegroundWindow() != hwnd:
            raise RuntimeError("game did not become foreground; capture refused")
        rect, origin = W.RECT(), W.POINT()
        if not user.GetClientRect(hwnd, C.byref(rect)) or not user.ClientToScreen(hwnd, C.byref(origin)):
            raise RuntimeError("cannot locate the owned client window")
        image = ImageGrab.grab(bbox=(origin.x, origin.y, origin.x + rect.right,
                                    origin.y + rect.bottom), all_screens=True)
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
