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
        const second = removed.splice(0, 1)[0];
        const third = actions.splice(0, 1)[0];
        const fourth = actions.splice(0, 1)[0];
        checksum = checksum + first() + second() + third() + fourth() + actions.length;
        i = i + 1;
    }
    return checksum;
}
function main() { console.log(workload(2, 7)); }
