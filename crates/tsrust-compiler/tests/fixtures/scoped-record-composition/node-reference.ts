
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
    const box: { items: Holder[] } = { items: [{ run: () => values[0] }] };
    box.items.push({ run: () => values[1] });
    strictEqual(box.items[0].run(), 3);
    strictEqual(box.items[1].run(), 7);
    const boxLast = box.items.pop();
    if (boxLast !== undefined) strictEqual(boxLast.run(), 7);
    const nestedBox: { groups: Holder[][] } = { groups: [[{ run: () => values[0] }]] };
    nestedBox.groups[0].push({ run: () => values[1] });
    const nestedBoxLast = nestedBox.groups[0].pop();
    nestedBox.groups[0][0] = { run: () => values[2] };
    if (nestedBoxLast !== undefined) strictEqual(nestedBoxLast.run(), 7);
    strictEqual(nestedBox.groups[0][0].run(), 11);
    const groups: Holder[][] = [[{ run: () => values[0] }], [{ run: () => values[1] }]];
    strictEqual(groups[0][0].run(), 3);
    strictEqual(groups[1][0].run(), 7);
    const left: Holder[] = [{ run: () => values[0] }];
    const right: Holder[] = [{ run: () => values[1] }];
    const storedGroups: Holder[][] = [left, right];
    strictEqual(storedGroups[0][0].run(), 3);
    strictEqual(storedGroups[1][0].run(), 7);
    const mutableGroups: Holder[][] = [[{ run: () => values[0] }]];
    mutableGroups[0].push({ run: () => values[1] });
    strictEqual(mutableGroups[0][0].run(), 3);
    strictEqual(mutableGroups[0][1].run(), 7);
    mutableGroups.push([{ run: () => values[2] }]);
    strictEqual(mutableGroups[1][0].run(), 11);
    const nestedLast = mutableGroups[0].pop();
    mutableGroups[0].push({ run: () => values[2] });
    const nestedRemoved = mutableGroups[0].splice(0, 1);
    mutableGroups[0].push({ run: () => values[0] });
    if (nestedLast !== undefined) strictEqual(nestedLast.run(), 7);
    strictEqual(nestedRemoved[0].run(), 3);
    mutableGroups[1] = [{ run: () => values[0] }];
    strictEqual(mutableGroups[1][0].run(), 3);
    mutableGroups[0][0] = { run: () => values[2] };
    strictEqual(mutableGroups[0][0].run(), 11);
    strictEqual(mutableGroups[0][1].run(), 3);
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
    const stack: Holder[] = [{ run: () => values[0] }];
    const last = stack.pop();
    const saved = last;
    stack.push({ run: () => values[1] });
    if (saved !== undefined) strictEqual(saved.run(), 3);
    const queue: Holder[] = [{ run: () => values[0] }];
    const first = queue.shift();
    queue.push({ run: () => values[1] });
    if (first !== undefined) strictEqual(first.run(), 3);
    let nullableOuter: Holder | undefined;
    {
        const source: Holder[] = [{ run: () => values[0] }];
        nullableOuter = source.shift();
    }
    if (nullableOuter !== undefined) strictEqual(nullableOuter.run(), 3);
}
storageComposition(true);
storageComposition(false);
