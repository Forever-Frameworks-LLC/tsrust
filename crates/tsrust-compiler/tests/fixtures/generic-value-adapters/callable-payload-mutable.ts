function make() {
    let calls = 0;
    function invoke<T extends () => number>(action: T): void {
        calls += 1;
        console.log(calls, action());
    }
    return invoke;
}

function main() {
    let count = 0;
    const action = (): number => { count += 1; return count; };
    const fixed: (action: () => number) => void = make();
    fixed(action);
    fixed(action);
    console.log(action());
}

main();
