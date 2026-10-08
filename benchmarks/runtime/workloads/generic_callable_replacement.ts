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
        output.push([original, selected, completed]);
    }
    return output;
}

function main() {
    const result = workload(3, 0, "replacement");
}
