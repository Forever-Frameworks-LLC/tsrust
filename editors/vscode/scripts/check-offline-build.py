#!/usr/bin/env python3
"""macOS acceptance for bundled stdlib builds with network access denied."""
import errno
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parents[3]
    compiler = root / "target/editor/debug/tsrust"
    evidence = root / "target/editor/offline-stdlib-evidence.json"
    sandbox = Path("/usr/bin/sandbox-exec")
    if sys.platform != "darwin" or not sandbox.exists():
        raise SystemExit("This network-denial acceptance check requires macOS sandbox-exec.")
    rustup = shutil.which("rustup")
    if not rustup or not compiler.exists():
        raise SystemExit("Build target/editor/debug/tsrust and install its pinned Rust backend first.")
    evidence.parent.mkdir(parents=True, exist_ok=True)
    # A failed run must not leave a previous successful result looking current.
    evidence.unlink(missing_ok=True)
    policy = "(version 1)(allow default)(deny network*)"

    def denied_run(arguments, environment=None, cwd=None):
        result = subprocess.run(
            [str(sandbox), "-p", policy, *map(str, arguments)],
            env=environment, cwd=cwd, capture_output=True, text=True, timeout=60,
        )
        if result.returncode:
            raise RuntimeError(f"Command failed: {arguments}\n{result.stderr}")
        return result.stdout

    probe = (
        "import errno,socket; s=socket.socket(); "
        "result=s.connect_ex(('127.0.0.1',9)); "
        "assert result in (errno.EPERM,errno.EACCES), result; print('network denied')"
    )
    assert denied_run([sys.executable, "-c", probe]).strip() == "network denied"
    with tempfile.TemporaryDirectory(prefix="offline-stdlib-", dir=evidence.parent) as temporary:
        directory = Path(temporary)
        wrappers = directory / "bin"
        wrappers.mkdir()
        wrapper = wrappers / "rustup"
        wrapper.write_text(
            '#!/bin/sh\n'
            'if [ "$1" != "run" ] || [ "$3" != "rustc" ]; then exit 90; fi\n'
            'if [ "$4" != "--version" ]; then printf "compile\\n" >> "$TSRUST_COMPILE_LOG"; fi\n'
            'exec "$TSRUST_REAL_RUSTUP" "$@"\n'
        )
        wrapper.chmod(0o755)
        environment = dict(os.environ)
        environment.update(
            PATH=str(wrappers) + os.pathsep + environment.get("PATH", ""),
            TSRUST_REAL_RUSTUP=rustup,
            TSRUST_COMPILE_LOG=str(directory / "compile.log"),
            TSRUST_CACHE_DIR=str(directory / "cache"),
        )
        dependency = directory / "helper.tsr"
        dependency.write_text(
            "import { basename } from 'std:path';"
            "export function name():string{return basename('/tmp/report.txt','.txt');}"
        )
        source = directory / "main.tsr"
        source.write_text("import { name } from './helper.tsr';function main(){console.log(name());}")
        for name in ("cold", "warm"):
            executable = directory / name
            denied_run([compiler, "build", source, "-o", executable], environment, directory)
            assert denied_run([executable], environment, directory) == "report\n"
        compilations = (directory / "compile.log").read_text().splitlines()
        assert compilations == ["compile"], compilations
        result = {
            "platform": sys.platform,
            "policy": policy,
            "network_denial_probe": "EPERM or EACCES connecting to loopback",
            "cold_build": "passed",
            "warm_build": "passed",
            "native_output": "report\n",
            "actual_compilation_invocations": len(compilations),
            "scope": "bundled std:path, installed pinned backend; no external packages",
        }
        evidence.write_text(json.dumps(result, indent=2) + "\n")
    print("Offline stdlib cold/warm builds passed; one actual compilation.")


if __name__ == "__main__":
    main()
