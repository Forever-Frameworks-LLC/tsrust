"""Remove stale tsrust native cache entries, keeping the newest ones.

Usage: python3 tools/prune-native-caches.py [--keep N] [--dry-run] [ROOT ...]

Every build keys its cache entries by content, so old entries are never reused
once sources or the compiler change, and nothing removes them. This keeps the
N most recently modified entries (default 2) in each namespace of every
`tsrust-native` directory and deletes the rest. With no ROOT, it searches the
repository for `target/tsrust-native` directories. Run it only while no
`tsrust build`, `dev` or `test` is running. `tsrust cache clean` removes every
entry instead.
"""
import argparse
import os
import shutil

NAMESPACES = ["v2", "runtime/v2", "runtime/build-v1", "incremental"]


def size(path):
    total = 0
    for base, _, files in os.walk(path):
        for name in files:
            try:
                total += os.lstat(os.path.join(base, name)).st_size
            except OSError:
                pass
    return total


def entries(path):
    names = [name for name in os.listdir(path) if len(name) == 16 and all(c in "0123456789abcdef" for c in name)]
    paths = [os.path.join(path, name) for name in names if os.path.isdir(os.path.join(path, name))]
    return sorted(paths, key=os.path.getmtime, reverse=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--keep", type=int, default=2)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("roots", nargs="*")
    arguments = parser.parse_args()
    repository = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    roots = arguments.roots
    if not roots:
        for base, directories, _ in os.walk(repository):
            if os.path.basename(base) == "target":
                roots += [os.path.join(base, name) for name in directories if name == "tsrust-native"]
                directories[:] = []
            else:
                directories[:] = [name for name in directories if name not in (".git", "node_modules")]
    freed = 0
    for root in roots:
        for namespace in NAMESPACES:
            path = os.path.join(root, namespace)
            if not os.path.isdir(path) or os.path.islink(path):
                continue
            stale = entries(path)[arguments.keep:]
            for entry in stale:
                freed += size(entry)
                if not arguments.dry_run:
                    shutil.rmtree(entry, ignore_errors=True)
            if stale:
                print(f"{os.path.relpath(path, repository)}: {'would remove' if arguments.dry_run else 'removed'} {len(stale)}")
    print(f"{'would free' if arguments.dry_run else 'freed'} {freed / 2**30:.1f} GiB")


if __name__ == "__main__":
    main()
