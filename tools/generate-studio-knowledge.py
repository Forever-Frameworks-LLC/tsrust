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
GUIDES = ["docs/guides/ui", *[f"website/src/content/docs/{area}" for area in ("language", "learn", "library", "start", "tools")]]
GPUI = [f"crates/tsrust-compiler/patched-dependencies/{crate}/src" for crate in ("gpui-base", "gpui-component")]
TESTS = ["crates/tsrust-compiler/tests/fixtures", "apps/studio/tests", "apps/studio/model-host/tests"]
SOURCES = [*GPUI, "examples", *GUIDES, *TESTS]


def passages(path, text):
    lines = text.splitlines()
    if path.suffix == ".rs":
        block = []
        for i, line in enumerate([*lines, ""]):
            comment = re.match(r"^\s*(?:///|//!) ?(.*)$", line)
            if comment:
                block.append(comment[1])
                continue
            if re.search(r"(?ms)^```(?:rust|ignore|no_run)?[ \t]*\n.+?^```[ \t]*$", "\n".join(block)):
                title = next((text for text in block if text.strip()), path.stem)
                yield i - len(block) + 1, title, "GPUI native Rust documentation:\n" + "\n".join(block)
            block = []
        return
    if path.suffix == ".md":
        titles = {}
        first = 0
        if lines and lines[0] == "---":
            first = lines.index("---", 1) + 1
            title = next((line[6:].strip() for line in lines[1:first] if line.startswith("title:")), path.stem)
            while first < len(lines) and not lines[first].strip():
                first += 1
            if first < len(lines):
                titles[first] = title
        fence = ""
        for i in range(first, len(lines)):
            marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", lines[i])
            if marker:
                run, tail = marker.groups()
                if not fence:
                    fence = run
                elif run.startswith(fence) and not tail.strip():
                    fence = ""
            elif not fence and re.match(r"^#{1,3} ", lines[i]):
                titles[i] = lines[i].strip()
        starts = list(titles)
    else:
        starts = [i for i, line in enumerate(lines) if re.match(
            r"^(?:(?:export|async) )*(?:function|class|interface|type|enum) |^(?:test|describe)\(", line
        )]
        # Module state and suite setup can precede the first named declaration.
        first = starts[0] if starts else len(lines)
        setup = next((i for i in range(first) if re.match(
            r"^(?:export )?(?:const|let) |^(?:beforeAll|afterAll|beforeEach|afterEach)\(", lines[i]
        )), None)
        if setup is not None:
            starts.insert(0, setup)
    for start, end in zip(starts, starts[1:] + [len(lines)]):
        title = titles[start] if path.suffix == ".md" else lines[start].strip()
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
            if path.suffix not in {".md", ".tsr", ".tsrx", ".rs"}:
                continue
            if path.suffix == ".rs" and not any(name.startswith(root + "/") for root in GPUI):
                continue
            if name == "apps/studio/tests/knowledge.test.tsr":
                continue  # Retrieval assertions contain expected answers and must not be searchable.
            if any(name.startswith(root + "/") for root in TESTS) and not name.endswith(".test.tsr"):
                continue
            if path.suffix == ".md" and not any(name.startswith(guide + "/") for guide in GUIDES):
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
