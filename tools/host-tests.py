#!/usr/bin/env python3
"""Run the Rust host tests: real process and language boundaries only.

Runtime language/library behavior lives in `tsrust test` suites (tools/run-suites.py).
These targets cover what a suite cannot: the CLI and its projects, test-runner
reporting, signals and child processes, relocated offline export, package
management, Rust adapters and the LSP stdio transport.

usage: host-tests.py [extra cargo test arguments, for example --features ...]
"""
import subprocess
import sys

COMPILER = [
    "acceptance", "project_commands", "formatting", "package_management", "toolchain_installation",
    "shared_sources", "native_dependency_bundle", "api_project", "api_rust_client", "router_contract",
    "service_sections", "process", "native_package_export", "rust_bridge", "rust_adapter_generation",
    "rust_data_boundaries", "testing_async", "test_reporter", "program_scoped_test_registry",
]
LSP = ["stdio", "api_project"]


def main():
    extra = sys.argv[1:]
    selection = [arg for name in COMPILER for arg in ("--test", name)]
    status = subprocess.run(["cargo", "test", "--offline", "--locked", "--no-fail-fast", "-p", "tsrust-compiler", *extra, *selection]).returncode
    selection = [arg for name in LSP for arg in ("--test", name)]
    status |= subprocess.run(["cargo", "test", "--offline", "--locked", "--no-fail-fast", "-p", "tsrust-lsp", *extra, *selection]).returncode
    sys.exit(1 if status else 0)


if __name__ == "__main__":
    main()
