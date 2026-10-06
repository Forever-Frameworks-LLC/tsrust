type int = number;
type FindRecord = { id: int; label: string; };
function workload(n: int, seed: int, input: string): string[] {
    const output: string[] = []; let i = 0;
    while (i < n) {
        const count = (i + seed) % 4;
        const mode = (i * 7 + seed) % 5;
        const values: FindRecord[] = [];
        if (count > 0) { values.push({ id: seed, label: input.repeat(1) }); }
        if (count > 1) { values.push({ id: seed + 1, label: input.concat("!") }); }
        if (count > 2) { values.push({ id: seed + 2, label: "é😀e\u0301\u0000" }); }
        let visits = 0; let trace = 0; let sourceVisits = 0;
        const selected = values.find((value: FindRecord, index: int, source: FindRecord[]): boolean => {
            visits = visits + 1;
            trace = trace * 10 + index + 1;
            sourceVisits = sourceVisits + source.length;
            return source.length === count && value.id === source[index].id && (mode === 1 && index === 0 || mode === 2 && index % 2 === 0 || mode === 3 && index >= 1 || mode === 4);
        });
        output.push(typeof selected);
        if (selected === undefined) { output.push("undefined"); } else { output.push(describe(selected)); }
        output.push(String(values.length)); output.push(String(visits)); output.push(String(trace)); output.push(String(sourceVisits));
        if (values.length > 0) { output.push(describe(values[0])); } else { output.push("<absent>"); }
        if (values.length > 1) { output.push(describe(values[1])); } else { output.push("<absent>"); }
        if (values.length > 2) { output.push(describe(values[2])); } else { output.push("<absent>"); }
        i = i + 1;
    }
    return output;
}
function describe(value: FindRecord): string { return String(value.id).concat(":", value.label); }
function main() { const result = workload(2, 7, "dynamic text"); }
