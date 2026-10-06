#!/usr/bin/env python3
"""Verify a copied CLI from an unrelated directory without installing it."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    root = Path(__file__).resolve().parents[3]
    name = "tsrust.exe" if os.name == "nt" else "tsrust"
    compiler = root / "target/editor/debug" / name
    evidence = root / "target/editor/cli-distribution-evidence.json"
    evidence.unlink(missing_ok=True)
    if not compiler.is_file():
        raise SystemExit("Build target/editor/debug/tsrust first.")
    if not shutil.which("rustup"):
        raise SystemExit("The pinned Rust backend must already be installed.")

    def run(arguments, environment, directory):
        result = subprocess.run(
            list(map(str, arguments)), cwd=directory, env=environment,
            capture_output=True, text=True, timeout=60,
        )
        if result.returncode:
            raise RuntimeError(f"Command failed: {arguments}\n{result.stderr}")
        return result.stdout

    with tempfile.TemporaryDirectory(prefix="tsrust distribution ") as temporary:
        directory = Path(temporary)
        binary = directory / "tools" / name
        binary.parent.mkdir()
        shutil.copy2(compiler, binary)
        project = directory / "unrelated project"
        project.mkdir()
        (project / "helper.tsr").write_text(
            'import { basename as fileName } from "std:path";'
            'export function report():string{return fileName("/tmp/report.txt",".txt");}'
        )
        source = project / "main.tsr"
        source.write_text(
            'import { report as name } from "./helper.tsr";'
            'function main(){console.log(name());}'
        )
        environment = dict(os.environ)
        environment["TSRUST_CACHE_DIR"] = str(directory / "cache")
        frontend = dict(environment, PATH="")
        run([binary, "check", source], frontend, project)
        emitted = run([binary, "emit", source], frontend, project)
        assert "tsrust_std_path_basename" in emitted
        executable = project / ("native program.exe" if os.name == "nt" else "native program")
        run([binary, "build", source, "-o", executable], environment, project)
        assert run([executable], environment, project) == "report\n"
        assert run([binary, "run", source], environment, project) == "report\n"
        result = {
            "platform": sys.platform,
            "compiler_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "frontend_without_path": ["check", "emit"],
            "copied_cli_build": "passed",
            "copied_cli_run": "passed",
            "native_output": "report\n",
            "scope": "copied debug CLI, unrelated project with spaces, installed pinned backend; no global installation or other-platform claim",
        }
        evidence.parent.mkdir(parents=True, exist_ok=True)
        evidence.write_text(json.dumps(result, indent=2) + "\n")
    print("Copied CLI check/emit/build/run passed outside the repository.")


if __name__ == "__main__":
    main()
