function make() {
    let calls = 0;
    function invoke<T extends () => number>(action: T): void { calls += 1; console.log(calls, action()); }
    return invoke;
}
function main() {
    const text = "owner";
    const action = (): number => text.length;
    const fixed: (action: () => number) => void = make();
    fixed(action);
    fixed(action);
    console.log(action());
}

main();
