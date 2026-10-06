"""Measure versioned stdio edit-to-diagnostic latency, excluding editor UI."""
import argparse
import json
import pathlib
import platform
import queue
import statistics
import subprocess
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument("server", type=pathlib.Path)
parser.add_argument("--debounce", type=int, default=200)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[3]
server = subprocess.Popen([str(args.server.resolve())], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
messages = queue.Queue()

def read():
    try:
        while True:
            length = None
            while line := server.stdout.readline():
                if line == b"\r\n":
                    break
                if line.lower().startswith(b"content-length:"):
                    length = int(line.split(b":", 1)[1])
            if length is None:
                return
            messages.put(json.loads(server.stdout.read(length)))
    except Exception as error:
        messages.put(error)

threading.Thread(target=read, daemon=True).start()

def send(method, params, request_id=None):
    message = {"jsonrpc": "2.0", "method": method, "params": params}
    if request_id is not None:
        message["id"] = request_id
    data = json.dumps(message, ensure_ascii=False).encode()
    server.stdin.write(f"Content-Length: {len(data)}\r\n\r\n".encode() + data)
    server.stdin.flush()

def receive():
    message = messages.get(timeout=10)
    if isinstance(message, Exception):
        raise message
    return message

try:
    send("initialize", {"capabilities": {}, "initializationOptions": {"debounceMs": args.debounce}}, 1)
    assert "result" in receive()
    send("initialized", {})
    fixtures = root / "crates/tsrust-compiler/tests/fixtures/ownership"
    large = "".join(f"function read{n}(value: int): int {{ return value + 1; }}\n" for n in range(1000)) + "function main() {}\n"
    rows = []
    for name, text in [("read-reuse", (fixtures / "read_reuse.tsr").read_text()), ("use-after-call", (fixtures / "use_after_call.tsr").read_text()), ("1000-functions", large)]:
        uri = f"file:///latency-{name}.tsr"
        send("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "tsrust", "version": 1, "text": text}})
        first = receive()
        assert first["params"]["version"] == 1
        expected_errors = len(first["params"]["diagnostics"])
        samples = []
        for version in range(2, 28):
            # Different unsaved text on every iteration; semantics stay constant.
            start = time.perf_counter()
            send("textDocument/didChange", {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text + f"// edit {version}\n"}]})
            message = receive()
            elapsed = (time.perf_counter() - start) * 1000
            assert message["method"] == "textDocument/publishDiagnostics"
            assert message["params"]["version"] == version
            assert len(message["params"]["diagnostics"]) == expected_errors
            if version >= 7:  # five warmup edits, 21 measured edits
                samples.append(elapsed)
        rows.append({"workload": name, "sourceBytes": len(text.encode()), "diagnostics": expected_errors, "medianMs": statistics.median(samples), "minMs": min(samples), "maxMs": max(samples), "samplesMs": samples})
        send("textDocument/didClose", {"textDocument": {"uri": uri}})
        assert receive()["params"]["diagnostics"] == []
    send("shutdown", None, 2)
    assert receive()["id"] == 2
    send("exit", None)
    server.stdin.close()
    assert server.wait(timeout=5) == 0
    print(json.dumps({"platform": platform.platform(), "server": str(args.server.resolve()), "debounceMs": args.debounce, "warmupEdits": 5, "measuredEdits": 21, "scope": "stdio full-text edit through versioned diagnostics, including debounce; excludes VS Code UI", "rows": rows}, indent=2))
finally:
    if server.poll() is None:
        server.kill()
        server.wait()
