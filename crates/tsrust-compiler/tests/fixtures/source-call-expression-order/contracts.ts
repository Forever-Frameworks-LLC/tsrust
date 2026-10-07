import { strictEqual } from "node:assert";
function identity<T>(value: T): T { return value; }
function forward<T>(value: T): T {
    let action = identity;
    action = (action = action, action);
    return (action = action)<T>(value);
}
function main() {
    strictEqual(forward(7), 7);
    strictEqual(forward("owned"), "owned");
    let count = 0;
    const first = identity<number>((count += 1, count));
    strictEqual(first, 1);
    false && (count += 10);
    true || (count += 100);
    const present = 0;
    present ?? (count += 1000);
    strictEqual(count, 1);
    const next = (count += 1, identity<number>(count));
    strictEqual(next, 2);
}

main();
