#!/usr/bin/env python3
"""Reconcile a fresh public_api_inventory output with explicit reviewed identities."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("inventory", type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
coverage = json.loads((root / "docs/plans/globals-and-imports-coverage.json").read_text())
inventory = json.loads(args.inventory.read_text())
expected = [row["identity"] for row in inventory["rows"]]
reviewed = []
for review, identities in coverage["reviews"].items():
    if not (root / review).is_file():
        raise SystemExit(f"Missing review: {review}")
    reviewed.extend(identities)
if len(expected) != len(set(expected)) or len(reviewed) != len(set(reviewed)):
    raise SystemExit("Duplicate inventory or review identity")
missing = sorted(set(expected) - set(reviewed))
stale = sorted(set(reviewed) - set(expected))
if missing or stale:
    raise SystemExit(f"Unreviewed: {missing}\nStale: {stale}")
if len(expected) != coverage["inventoryRows"]:
    raise SystemExit("Recorded inventory count is stale")
print(f"PASS: {len(expected)} identities have exactly one review; discovery completeness remains a separate source review.")
