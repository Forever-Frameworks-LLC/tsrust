"""Build Studio's embedded SQLite FTS corpus from tracked public examples and user guides."""
import base64
import hashlib
import json
import re
import sqlite3
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "apps/studio/src/assets/knowledge"
SOURCES = ["examples", "docs/guides/ui", "crates/tsrust-compiler/tests/fixtures"]


def passages(path, text):
    lines = text.splitlines()
    if path.suffix == ".md":
        starts = [i for i, line in enumerate(lines) if re.match(r"^#{1,3} ", line)]
    else:
        starts = [i for i, line in enumerate(lines) if re.match(
            r"^(?:(?:export|async) )*(?:function|class|interface|type|enum) |^(?:test|describe)\(", line
        )]
    for start, end in zip(starts, starts[1:] + [len(lines)]):
        title = lines[start].strip()
        if path.suffix == ".tsrx" and path.parent.name == "pages" and title.startswith("export "):
            continue  # Gallery page scaffolding repeats its examples as quoted source.
        body = "\n".join(lines[start:end]).rstrip()
        if body and (path.suffix != ".md" or body != title):
            yield start + 1, title, body


def main():
    files = subprocess.check_output(["git", "ls-files", "-z", "--", *SOURCES], cwd=ROOT)
    with tempfile.TemporaryDirectory() as temporary:
        database = Path(temporary) / "examples.sqlite"
        db = sqlite3.connect(database)
        db.execute("CREATE VIRTUAL TABLE passages USING fts5(path, title, terms, body, line UNINDEXED)")
        for name in sorted(files.decode().split("\0")):
            path = Path(name)
            if name == "examples/gpui-starter/src/search-index.tsr":
                continue  # Generated gallery navigation data is not a code example.
            if path.suffix not in {".md", ".tsr", ".tsrx"}:
                continue
            if name.startswith("crates/") and not name.endswith(".test.tsr"):
                continue
            if path.suffix == ".md" and not name.startswith("docs/guides/ui/"):
                continue
            for line, title, body in passages(path, (ROOT / path).read_text()):
                terms = re.sub(r"([a-z])([A-Z])", r"\1 \2", path.stem + " " + title)
                db.execute("INSERT INTO passages VALUES (?,?,?,?,?)", (name, title, terms, body, line))
        db.execute("INSERT INTO passages(passages) VALUES('optimize')")
        db.commit()
        db.execute("VACUUM")
        count, sources = db.execute("SELECT count(*),count(DISTINCT path) FROM passages").fetchone()
        db.close()
        data = database.read_bytes()
    DEST.mkdir(parents=True, exist_ok=True)
    (DEST / "passages.txt").write_text(base64.b64encode(data).decode())
    info = dict(sha256=hashlib.sha256(data).hexdigest(), size=len(data))
    (DEST / "manifest.json").write_text(json.dumps(info, indent=2) + "\n")
    print(f"{count} passages from {sources} files, {len(data)} SQLite bytes")


if __name__ == "__main__":
    main()
