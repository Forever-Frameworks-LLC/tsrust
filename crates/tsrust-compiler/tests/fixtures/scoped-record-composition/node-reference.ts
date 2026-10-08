
        import { strictEqual } from "node:assert";
        type Holder = { run: () => number; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [
                    { run: (): number => values[0] },
                    { run: (): number => values[1] }
                ];
                strictEqual(holders[0].run(), 3);
                strictEqual(holders[1].run(), 7);
                for (const holder of holders) strictEqual(holder.run() > 0, true);
                const last = holders.pop();
                if (last !== undefined) strictEqual(last.run(), 7);
                const first = holders.shift();
                if (first !== undefined) strictEqual(first.run(), 3);
                strictEqual(holders.length, 0);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    
main();

function storageComposition(flag: boolean) {
    const values = [3, 7, 11];
    const initial: Holder[] = [{ run: () => values[0] }];
    const moved = initial;
    const removed = moved.splice(0, 1);
    if (flag) moved.push({ run: () => values[1] });
    else moved.push({ run: () => values[2] });
    strictEqual(removed[0].run(), 3);
    strictEqual(moved[0].run(), flag ? 7 : 11);
    let outer: Holder[];
    {
        const holders: Holder[] = [{ run: () => values[0] }];
        outer = holders.splice(0, 1);
    }
    strictEqual(outer[0].run(), 3);
}
storageComposition(true);
storageComposition(false);
