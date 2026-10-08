
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
