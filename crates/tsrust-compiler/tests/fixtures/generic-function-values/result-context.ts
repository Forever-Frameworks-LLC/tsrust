function empty<T>(): T[] { return []; }
function make(start: number) {
    let calls = start;
    function fresh<T>(): T[] { calls += 1; console.log(calls); return []; }
    return fresh;
}
function main() {
    const plain = empty;
    const plainNumbers: number[] = plain();
    const plainStrings: string[] = plain();
    plainNumbers.push(7);
    plainStrings.push("plain");
    if (plainNumbers[0] !== 7 || plainStrings[0] !== "plain") { throw 99; }
    const action = make(10);
    const numbers: number[] = action();
    const strings: string[] = action();
    numbers.push(3);
    strings.push("x");
    console.log(numbers[0], strings[0]);
}

main();
