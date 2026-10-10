type int = number;
type User = { value: int; };
type Result<T, E> = { kind: "Ok"; value: T } | { kind: "Err"; error: E };

function make(value: int): Result<User, int> {
    if (value % 3 === 0) { return { kind: "Err", error: value + 5 }; }
    return { kind: "Ok", value: { value } };
}

function inspect(result: Result<User, int>): int {
    switch (result.kind) {
        case "Ok": return result.value.value * 2;
        case "Err": return result.error * 3;
    }
}

function counter(start: int): Result<() => int, int> {
    if (start % 4 === 0) { return { kind: "Err", error: start }; }
    let value = start;
    return { kind: "Ok", value: (): int => { value += 1; return value; } };
}

function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 17 + seed % 19 + 1;
        const result = make(value);
        const first = inspect(result);
        const second = inspect(result);
        let owned = 0;
        switch (result.kind) {
            case "Ok": owned = result.value.value + 7; break;
            case "Err": owned = result.error + 11; break;
        }
        const callback = counter(value);
        let callA = 0;
        let callB = 0;
        switch (callback.kind) {
            case "Ok": callA = callback.value(); callB = callback.value(); break;
            case "Err": callA = callback.error; callB = callback.error + 1; break;
        }
        output.push([first, second, owned, callA, callB]);
        i += 1;
    }
    return output;
}

function main() { const result = workload(3, 0, "results"); }
