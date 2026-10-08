import { strictEqual } from "node:assert";
type Counts = { calls: number; };
function next(counts: Counts): string { counts.calls += 1; return "next"; }
function main() {
    const counts: Counts = { calls: 0 };
    let text = "kept";
    text ||= next(counts);
    strictEqual(counts.calls, 0);
    text &&= next(counts);
    strictEqual(counts.calls, 1);
    strictEqual(text, "next");
    text = "";
    text &&= next(counts);
    strictEqual(counts.calls, 1);
    text ||= next(counts);
    strictEqual(counts.calls, 2);
    strictEqual(text, "next");
    strictEqual(!text, false);
    strictEqual(!!text, true);
    let absent: string | undefined = undefined;
    absent ||= "present";
    if (absent) strictEqual(absent.length, 7);
    else strictEqual(true, false);
    const values: number[] = [];
    strictEqual(!values, false);
    strictEqual(!NaN, true);
    strictEqual(!0, true);
    strictEqual(!-0.0, true);
    strictEqual(!" ", false);
}

main();
