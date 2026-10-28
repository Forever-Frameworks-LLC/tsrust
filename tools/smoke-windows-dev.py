"""Exercise native Windows rebuild, failed-edit recovery and console shutdown."""
import ctypes
from ctypes import wintypes
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

if os.name != "nt":
    raise SystemExit("This check requires Windows")
compiler = str(Path(sys.argv[1]).resolve(strict=True))
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
kernel.OpenProcess.restype = wintypes.HANDLE
kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
kernel.CloseHandle.argtypes = [wintypes.HANDLE]
kernel.AttachConsole.argtypes = [wintypes.DWORD]
kernel.GenerateConsoleCtrlEvent.argtypes = [wintypes.DWORD, wintypes.DWORD]
kernel.SetConsoleCtrlHandler.argtypes = [ctypes.c_void_p, wintypes.BOOL]


def wait_text(log, text, process, seconds=90, starts=0, occurrences=1):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        output = log.read_text(encoding="utf-8", errors="replace")
        if output.count(text) >= occurrences and output.count("Started program (pid ") >= starts:
            return output
        assert process.poll() is None, output
        time.sleep(0.05)
    raise AssertionError(f"Missing {text}: {output}")


def interrupt(process, event=1):
    # Attach this disposable Python process to the hidden test console, never
    # the user's terminal. Send Ctrl+Break only to the supervisor's process group.
    kernel.FreeConsole()
    assert kernel.AttachConsole(process.pid), ctypes.get_last_error()
    # Ctrl+C is console-wide. Ignore it in this controller only, after the
    # supervisor has started, so the test can verify its real handler and exit.
    if event == 0:
        assert kernel.SetConsoleCtrlHandler(None, True), ctypes.get_last_error()
    try:
        assert kernel.GenerateConsoleCtrlEvent(event, 0 if event == 0 else process.pid), ctypes.get_last_error()
        assert process.wait(timeout=5) == 0
    finally:
        if event == 0:
            kernel.SetConsoleCtrlHandler(None, False)
        kernel.FreeConsole()


def backend_project(root):
    """Check actual generated HTTP development without replacing owner edits."""
    import json
    import re
    import socket
    import urllib.request
    import urllib.error

    root = root.resolve(strict=True)
    main = (root / "app/main.tsr").read_text(encoding="utf-8")
    assert "api:server" in main
    assert "app.listen('127.0.0.1:4318')" in main, "Expected the default review service address"
    config = json.loads((root / "tsrust.config.json").read_text(encoding="utf-8"))
    assert config["build"]["entry"] == "app/main.tsr"
    entry = root / "app/lib/greeting.tsr"
    original = entry.read_bytes()
    patched = original.replace(b"Hello, ", b"Rebuilt, ")
    assert patched != original, "Expected the untouched greeting scaffold"
    invalid = b"export function greet(name: string): string { let broken: int = ; }\n"
    # Refuse to connect to or interrupt an owner's already-running service.
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 4318))
    log = Path(__file__).resolve().parent.parent / "target/api-server-dev-current.log"
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    handles = []
    with log.open("wb") as output:
        process = subprocess.Popen(
            [compiler, "dev", "--verbose", "--color=never"], cwd=root,
            stdout=output, stderr=output, startupinfo=startup,
            creationflags=subprocess.CREATE_NEW_CONSOLE | subprocess.CREATE_NEW_PROCESS_GROUP,
            env={**os.environ, "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1"},
        )
        try:
            first = wait_text(log, "API ready", process, seconds=240, starts=1)
            pid = int(re.findall(r"Started program \(pid (\d+)\)", first)[-1])
            handle = kernel.OpenProcess(0x00100000, False, pid)
            assert handle, ctypes.get_last_error()
            handles.append(handle)
            with urllib.request.urlopen("http://127.0.0.1:4318/api/description", timeout=5) as response:
                description = json.load(response)
            contract = description["history"]["sdkReleases"]["0.1.0"]["contractId"]

            def expect_greeting(expected):
                deadline = time.monotonic() + 10
                observed = None
                while time.monotonic() < deadline:
                    request = urllib.request.Request(
                        "http://127.0.0.1:4318/api", method="POST",
                        data=json.dumps({"operation": "greet", "input": "dev"}).encode(),
                        headers={"content-type": "application/json", "x-tsrust-contract": contract},
                    )
                    try:
                        with urllib.request.urlopen(request, timeout=2) as response:
                            observed = json.load(response)
                        if observed == {"ok": True, "value": expected}:
                            return
                    except (urllib.error.URLError, TimeoutError):
                        pass
                    assert process.poll() is None, log.read_text(encoding="utf-8")
                    time.sleep(0.05)
                raise AssertionError(f"Expected {expected}, received {observed}")

            expect_greeting("Hello, dev!")
            assert entry.read_bytes() == original, "Owner changed the fixture before our edit"
            entry.write_bytes(invalid)
            wait_text(log, "previous application remains running", process)
            assert kernel.WaitForSingleObject(handle, 0) == 258
            expect_greeting("Hello, dev!")
            assert entry.read_bytes() == invalid, "Owner changed the failed edit"
            entry.write_bytes(patched)
            subprocess.run([compiler, "format", "--write", str(entry)], check=True, capture_output=True)
            subprocess.run([compiler, "format", "--check", str(entry)], check=True, capture_output=True)
            rebuilt = wait_text(log, "API ready", process, seconds=180, starts=2, occurrences=2)
            assert "Rebuilt in" in rebuilt
            assert kernel.WaitForSingleObject(handle, 3000) == 0
            next_pid = int(re.findall(r"Started program \(pid (\d+)\)", rebuilt)[-1])
            next_handle = kernel.OpenProcess(0x00100000, False, next_pid)
            assert next_handle, ctypes.get_last_error()
            handles.append(next_handle)
            expect_greeting("Rebuilt, dev!")
            assert not (root / "generated").exists()
            assert not (root / "target/tsrust-api/server.tsr").exists()
            interrupt(process)
            assert "[dev] Stopped." in log.read_text(encoding="utf-8")
            assert kernel.WaitForSingleObject(next_handle, 3000) == 0
        finally:
            if process.poll() is None:
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True)
                process.wait(timeout=5)
            for handle in handles:
                kernel.CloseHandle(handle)
            current = entry.read_bytes()
            if current in (patched, invalid):
                entry.write_bytes(original)
            else:
                assert current == original, "Concurrent owner edit preserved; fixture not restored"
    print("Backend dev: live HTTP, failed-edit preservation, hot reload, no generated source, restored fixture and owned-process cleanup passed.")


if len(sys.argv) > 2:
    assert len(sys.argv) == 4 and sys.argv[2] == "--backend-project"
    backend_project(Path(sys.argv[3]))
    raise SystemExit(0)


with tempfile.TemporaryDirectory(prefix="tsrust-windows-dev-") as directory:
    parent = Path(directory) / "space \u03bb"
    parent.mkdir()
    root = parent / "app"
    subprocess.run([compiler, "new", str(root)], check=True, capture_output=True)
    entry = root / "src/main.tsr"

    def write(text):
        entry.write_text(text, encoding="utf-8")
        subprocess.run([compiler, "format", str(root)], check=True, capture_output=True)
        subprocess.run([compiler, "format", str(root), "--check"], check=True, capture_output=True)

    # Keep the first application alive so replacement/failure behavior is observable.
    write('function main(){console.log("windows-first");while(true){}}')
    log = root / "dev-output.log"
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 0
    with log.open("wb") as output:
        process = subprocess.Popen(
            [compiler, "dev", "--verbose", "--color=never"], cwd=root,
            stdout=output, stderr=output, startupinfo=startup,
            creationflags=subprocess.CREATE_NEW_CONSOLE | subprocess.CREATE_NEW_PROCESS_GROUP,
            env={**os.environ, "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1"},
        )
        handles = []
        try:
            first = wait_text(log, "windows-first", process, starts=1)
            import re
            pid = int(re.search(r"Started program \(pid (\d+)\)", first)[1])
            handle = kernel.OpenProcess(0x00100000, False, pid)
            assert handle, ctypes.get_last_error()
            handles.append(handle)
            entry.write_text("function main(){let broken: int = ;}", encoding="utf-8")
            wait_text(log, "previous application remains running", process)
            assert kernel.WaitForSingleObject(handle, 0) == 258, "last good application stopped on failed edit"
            write('function main(){console.log("windows-second");while(true){}}')
            rebuilt = wait_text(log, "windows-second", process, starts=2)
            assert kernel.WaitForSingleObject(handle, 3000) == 0, "old application survived replacement"
            next_pid = int(re.findall(r"Started program \(pid (\d+)\)", rebuilt)[-1])
            next_handle = kernel.OpenProcess(0x00100000, False, next_pid)
            assert next_handle, ctypes.get_last_error()
            handles.append(next_handle)
            interrupt(process)
            assert "[dev] Stopped." in log.read_text(encoding="utf-8"), log.read_text()
            assert kernel.WaitForSingleObject(next_handle, 3000) == 0, "application survived supervisor shutdown"
        finally:
            if process.poll() is None:
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True)
                process.wait(timeout=5)
            for handle in handles:
                kernel.CloseHandle(handle)
    with log.open("wb") as output:
        process = subprocess.Popen(
            [compiler, "dev", "--verbose", "--color=never"], cwd=root,
            stdout=output, stderr=output, startupinfo=startup,
            creationflags=subprocess.CREATE_NEW_CONSOLE | subprocess.CREATE_NEW_PROCESS_GROUP,
            env={**os.environ, "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1"},
        )
        try:
            wait_text(log, "windows-second", process, starts=1)
            interrupt(process, event=0)
            assert "[dev] Stopped." in log.read_text(encoding="utf-8")
        finally:
            if process.poll() is None:
                subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True)
                process.wait(timeout=5)
    print("Windows dev: Unicode/space path, native rebuild, failed-edit preservation, replacement, Ctrl+C and Ctrl+Break cleanup passed.")
