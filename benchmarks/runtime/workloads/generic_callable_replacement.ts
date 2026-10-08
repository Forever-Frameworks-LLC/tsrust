type int = number;
function first<T>(value: T, base: int): int {
    return base;
}

function second<T>(value: T, base: int): int {
    return base + 7;
}

function failing<T>(value: T, base: int): int {
    throw "selected failure";
}

function attempt(flag: boolean, base: int): int {
    let action = first;
    if (flag) { action = failing; }
    try { return action<int>(base, base); }
    catch (error) {
        if (error === "selected failure") { return 11; }
        return 0;
    }
}

function firstDefault<T>(value: T, offset: int = 3): int { return offset; }
function secondDefault<T>(value: T, offset: int = 7): int { return offset; }
function defaulted(flag: boolean): int {
    let action = firstDefault;
    if (flag) { action = secondDefault; }
    return action<int>(1);
}
function ownedHead<T>(value: T, ...tail: T[]): T { return value; }
function ownedTail<T>(value: T, ...tail: T[]): T { return tail.shift()!; }
function selectedText(flag: boolean): string {
    let action = ownedHead;
    if (flag) { action = ownedTail; }
    return action<string>("head", "next-value");
}

function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    for (let i = 0; i < n; i += 1) {
        const base = i % 5 + seed % 5 + 1;
        const flag = (i + seed) % 2 === 1;
        let action = first;
        const saved = action;
        action = first;
        if (flag) { action = second; }
        const original = saved<int>(base, base);
        const selected = action<int>(base, base);
        const completed = attempt(flag, base);
        const omitted = defaulted(flag);
        const text = selectedText(flag);
        output.push([original, selected, completed, omitted, text.length]);
    }
    return output;
}

function main() {
    const result = workload(3, 0, "replacement");
}
