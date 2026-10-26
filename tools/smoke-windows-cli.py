"""Check the real CLI in isolated hidden Windows consoles, without a terminal UI."""
import ctypes
from ctypes import wintypes as w
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

if os.name != "nt":
    raise SystemExit("This check requires Windows")
compiler = str(Path(sys.argv[1]).resolve(strict=True))
if "--console-host" not in sys.argv:
    startup = subprocess.STARTUPINFO()
    startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    process = subprocess.Popen(
        [sys.executable, "-B", __file__, compiler, "--console-host"],
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        startupinfo=startup, creationflags=subprocess.CREATE_NEW_CONSOLE,
    )
    try:
        output, errors = process.communicate(timeout=90)
    except subprocess.TimeoutExpired:
        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True)
        process.wait(timeout=5)
        raise
    sys.stdout.write(output.decode("utf-8", errors="replace"))
    sys.stderr.write(errors.decode("utf-8", errors="replace"))
    raise SystemExit(process.returncode)

import _winapi
k = ctypes.WinDLL("kernel32", use_last_error=True)


class Coord(ctypes.Structure):
    _fields_ = [("x", w.SHORT), ("y", w.SHORT)]


class Rect(ctypes.Structure):
    _fields_ = [("left", w.SHORT), ("top", w.SHORT), ("right", w.SHORT), ("bottom", w.SHORT)]


class Info(ctypes.Structure):
    _fields_ = [("size", Coord), ("cursor", Coord), ("attributes", w.WORD), ("window", Rect), ("maximum", Coord)]


class Key(ctypes.Structure):
    _fields_ = [("down", w.BOOL), ("repeat", w.WORD), ("key", w.WORD), ("scan", w.WORD), ("char", w.WCHAR), ("control", w.DWORD)]


class Event(ctypes.Union):
    _fields_ = [("key", Key), ("padding", ctypes.c_byte * 16)]


class Input(ctypes.Structure):
    _fields_ = [("kind", w.WORD), ("event", Event)]


class Cursor(ctypes.Structure):
    _fields_ = [("size", w.DWORD), ("visible", w.BOOL)]


k.AttachConsole.argtypes = [w.DWORD]
k.CreateFileW.argtypes = [w.LPCWSTR, w.DWORD, w.DWORD, ctypes.c_void_p, w.DWORD, w.DWORD, w.HANDLE]
k.CreateFileW.restype = w.HANDLE
k.GetConsoleScreenBufferInfo.argtypes = [w.HANDLE, ctypes.POINTER(Info)]
k.ReadConsoleOutputCharacterW.argtypes = [w.HANDLE, w.LPWSTR, w.DWORD, Coord, ctypes.POINTER(w.DWORD)]
k.WriteConsoleInputW.argtypes = [w.HANDLE, ctypes.POINTER(Input), w.DWORD, ctypes.POINTER(w.DWORD)]
k.SetConsoleWindowInfo.argtypes = [w.HANDLE, w.BOOL, ctypes.POINTER(Rect)]
k.GetConsoleMode.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
k.GetConsoleCursorInfo.argtypes = [w.HANDLE, ctypes.POINTER(Cursor)]
k.SetConsoleCtrlHandler.argtypes = [ctypes.c_void_p, w.BOOL]
k.CloseHandle.argtypes = [w.HANDLE]
k.ResumeThread.argtypes = [w.HANDLE]
k.ResumeThread.restype = w.DWORD
k.GetConsoleProcessList.argtypes = [ctypes.POINTER(w.DWORD), w.DWORD]
k.SetHandleInformation.argtypes = [w.HANDLE, w.DWORD, w.DWORD]
k.FillConsoleOutputCharacterW.argtypes = [w.HANDLE, w.WCHAR, w.DWORD, Coord, ctypes.POINTER(w.DWORD)]
k.SetConsoleCursorPosition.argtypes = [w.HANDLE, Coord]


class Console:
    def __init__(self, root, term, args=()):
        self.process = self.thread = None
        self.input = self.output = None
        try:
            clients = (w.DWORD * 8)()
            assert k.GetConsoleProcessList(clients, 8) == 1 and clients[0] == os.getpid(), "test must own an isolated console"
            self.input = k.CreateFileW("CONIN$", 0xC0000000, 3, None, 3, 0, None)
            self.output = k.CreateFileW("CONOUT$", 0xC0000000, 3, None, 3, 0, None)
            assert self.input not in (None, ctypes.c_void_p(-1).value)
            assert self.output not in (None, ctypes.c_void_p(-1).value)
            info = Info()
            assert k.GetConsoleScreenBufferInfo(self.output, ctypes.byref(info))
            written = w.DWORD()
            assert k.FillConsoleOutputCharacterW(self.output, " ", info.size.x * info.size.y, Coord(0, 0), ctypes.byref(written))
            assert k.SetConsoleCursorPosition(self.output, Coord(0, 0))
            assert k.SetConsoleWindowInfo(self.output, True, ctypes.byref(Rect(0, 0, 79, 13))), ctypes.get_last_error()
            mode = w.DWORD()
            assert k.GetConsoleMode(self.input, ctypes.byref(mode))
            self.original = mode.value
            for handle in (self.input, self.output):
                assert k.SetHandleInformation(handle, 1, 1)
            startup = subprocess.STARTUPINFO()
            startup.dwFlags = subprocess.STARTF_USESTDHANDLES
            startup.hStdInput = self.input
            startup.hStdOutput = startup.hStdError = self.output
            env = {**os.environ, "TERM": term, "NO_COLOR": "1"}
            self.process, self.thread, _, _ = _winapi.CreateProcess(
                compiler, subprocess.list2cmdline([compiler, *args]), None, None, True,
                0x4, env, str(root), startup,
            )
            assert k.SetConsoleCtrlHandler(None, True)
            # Configure viewport and record the original mode before the CLI runs.
            assert k.ResumeThread(self.thread) != 0xFFFFFFFF, ctypes.get_last_error()
            _winapi.CloseHandle(self.thread)
            self.thread = None
        except BaseException:
            self.close()
            raise

    def text(self):
        info = Info()
        assert k.GetConsoleScreenBufferInfo(self.output, ctypes.byref(info)), ctypes.get_last_error()
        first = max(0, info.cursor.y - 60)
        count = info.size.x * min(80, info.size.y - first)
        buffer = ctypes.create_unicode_buffer(count)
        read = w.DWORD()
        assert k.ReadConsoleOutputCharacterW(self.output, buffer, count, Coord(0, first), ctypes.byref(read)), ctypes.get_last_error()
        return "\n".join(buffer[i:i + info.size.x].rstrip() for i in range(0, read.value, info.size.x))

    def expect(self, text):
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline:
            output = self.text()
            if text in output:
                return output
            assert _winapi.WaitForSingleObject(self.process, 0) != 0, output
            time.sleep(0.02)
        raise AssertionError(f"Missing {text}: {output}")

    def key(self, virtual, char="\0", control=0):
        events = (Input * 2)()
        for event, down in zip(events, [True, False]):
            event.kind = 1
            event.event.key = Key(down, 1, virtual, 0, char, control)
        written = w.DWORD()
        assert k.WriteConsoleInputW(self.input, events, 2, ctypes.byref(written)) and written.value == 2

    def finish(self, mode=None):
        assert _winapi.WaitForSingleObject(self.process, 5000) == 0, self.text()
        assert _winapi.GetExitCodeProcess(self.process) == 0, self.text()
        current = w.DWORD()
        assert k.GetConsoleMode(self.input, ctypes.byref(current))
        if mode is not None:
            assert current.value == mode, (current.value, mode)
        cursor = Cursor()
        assert k.GetConsoleCursorInfo(self.output, ctypes.byref(cursor)) and cursor.visible
        return current.value

    def close(self):
        if self.process is not None:
            if _winapi.WaitForSingleObject(self.process, 0) != 0:
                _winapi.TerminateProcess(self.process, 1)
                _winapi.WaitForSingleObject(self.process, 5000)
            _winapi.CloseHandle(self.process)
        if self.thread is not None:
            _winapi.CloseHandle(self.thread)
        for handle in (self.input, self.output):
            if handle not in (None, ctypes.c_void_p(-1).value):
                k.CloseHandle(handle)
        k.SetConsoleCtrlHandler(None, False)


with tempfile.TemporaryDirectory(prefix="tsrust-windows-cli-") as directory:
    root = Path(directory)
    assert root.resolve().parent == Path(tempfile.gettempdir()).resolve()
    assert root.name.startswith("tsrust-windows-cli-")
    terminal = Console(root, "dumb")
    try:
        terminal.expect("Choose a number")
        terminal.key(0x51, "q")
        terminal.key(0x0D, "\r")
        terminal.finish(terminal.original)
    finally:
        terminal.close()
    for control_c in [False, True]:
        terminal = Console(root, "xterm-256color", ["--color=never"])
        try:
            text = terminal.expect("What would you like to do?")
            assert "\u2580\u2588\u2580" in text, text
            assert "Choose a number" not in text, text
            if control_c:
                terminal.key(0x43, "\x03", 8)
            else:
                terminal.key(0x28)
                terminal.key(0x0D, "\r")
                terminal.expect("Choose a destination")
                terminal.key(0x1B, "\x1b")
            terminal.finish(terminal.original)
        finally:
            terminal.close()
    assert not list(root.iterdir()), "cancelled launcher must not write project files"
print("Windows CLI: plain fallback, small-panel branding, arrow/Enter/Esc, Ctrl+C, cursor and input-mode restoration passed.")
