function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const base = i + seed;
        const actions: (() => number)[] = [(): number => { return base; }];
        actions.push((): number => { return base + 1; });
        actions.unshift((): number => { return base + 2; });
        actions[1] = (): number => { return base + 3; };
        const first = actions.splice(0, 1)[0];
        const removed = actions.splice(0, 1, (): number => { return base + 4; });
        checksum = checksum + first() + removed[0]() + actions[0]() + actions[1]() + actions.length;
        i = i + 1;
    }
    return checksum;
}
function main() { console.log(workload(2, 7)); }
