mod support;

fn project(source: &str) {
    let source = support::format_round_trip(source);
    let directory = support::TempDirectory::new();
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn scoped_conditional_callbacks_pass_through_array_map_and_for_each() {
    project(r#"
        import { strictEqual } from "std:testing";
        function main() {
            const offsets = [3, 7];
            const values = [1, 2];
            const flag = values.length === 2;
            const selected = flag
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const mapped = values.map(selected);
            strictEqual(mapped[0], 4);
            strictEqual(mapped[1], 5);
            strictEqual(offsets[1], 7);
            let left = 0;
            let right = 0;
            {
                const observe = flag
                    ? (value: int): void => { left += value; }
                    : (value: int): void => { right += value; };
                values.forEach(observe);
            }
            strictEqual(left, 3);
            strictEqual(right, 0);
        }
    "#);
}

#[test]
fn scoped_conditional_callbacks_keep_nullable_optional_invocation_and_result_fallback() {
    project(r#"
        import { strictEqual } from "std:testing";
        function main() {
            const offsets = [3, 7];
            const flag = offsets.length === 2;
            const selected = flag
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const optional = flag ? selected : null;
            strictEqual(typeof optional, "function");
            strictEqual(optional?.(2) ?? -1, 5);
            strictEqual(optional?.(4) ?? -1, 7);
            const absentSelection = flag
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const missing = !flag ? absentSelection : null;
            strictEqual(typeof missing, "object");
            strictEqual(missing?.(9) ?? -1, -1);
            strictEqual(offsets[0], 3);

            const scopedValues = [1, 2];
            {
                const local = flag ? (): int => scopedValues[0] : (): int => scopedValues[1];
                const localOptional = flag ? local : null;
                strictEqual(localOptional?.() ?? 0, 1);
            }
            scopedValues[0] = 9;
            strictEqual(scopedValues[0], 9);
        }
    "#);
}

#[test]
fn scoped_conditional_callbacks_preserve_reduction_predicate_sort_and_flat_map_contracts() {
    project(r#"
        import { strictEqual } from "std:testing";
        function main() {
            const offsets = [3, 7];
            const flag = offsets.length === 2;
            const values = [1, 2, 3];
            const predicate = flag
                ? (value: int): boolean => value < offsets[0]
                : (value: int): boolean => value < offsets[1];
            strictEqual(values.every(predicate), false);
            strictEqual(values.some(predicate), true);
            strictEqual(values.find(predicate) ?? -1, 1);
            strictEqual(values.findIndex(predicate), 0);
            strictEqual(values.filter(predicate).length, 2);
            const reducer = flag
                ? (sum: int, value: int): int => sum + value + offsets[0]
                : (sum: int, value: int): int => sum + value + offsets[1];
            strictEqual(values.reduce(reducer, 0), 15);
            strictEqual(values.reduceRight(reducer, 0), 15);
            const comparator = flag
                ? (a: int, b: int): int => (a - b) * offsets[0]
                : (a: int, b: int): int => (a - b) * offsets[1];
            const unordered = [3, 1, 2];
            const sorted = unordered.toSorted(comparator);
            strictEqual(sorted[0], 1);
            strictEqual(unordered[0], 3);
            unordered.sort(comparator);
            strictEqual(unordered[2], 3);
            const flatten = flag
                ? (value: int): int[] => [value, value + offsets[0]]
                : (value: int): int[] => [value, value + offsets[1]];
            const flattened = values.flatMap(flatten);
            strictEqual(flattened.length, 6);
            strictEqual(flattened[5], 6);
            const mapper = flag
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const copied = Array.from(values, mapper);
            strictEqual(copied[2], 6);
            const defaulted = flag
                ? (value: int, index: int, source: ReadOnly<int[]>, extra: int = 4): int => value + index + source.length + extra + offsets[0]
                : (value: int, index: int, source: ReadOnly<int[]>, extra: int = 6): int => value + index + source.length + extra + offsets[1];
            const defaultResults = values.map(defaulted);
            strictEqual(defaultResults[0], 11);
            strictEqual(defaultResults[2], 15);
            const restComparator = flag
                ? (...pair: int[]): int => (pair[0] - pair[1]) * offsets[0]
                : (...pair: int[]): int => (pair[0] - pair[1]) * offsets[1];
            const restSorted = values.toSorted(restComparator);
            strictEqual(restSorted[0], 1);
        }
    "#);
}

#[test]
fn scoped_conditional_callbacks_preserve_collection_traversal_and_failure_payloads() {
    project(r#"
        import { strictEqual } from "std:testing";
        function main() {
            const offsets = [3, 7];
            const flag = offsets.length === 2;
            const values = [1, 2];
            const failing = flag
                ? (value: int): int => { if (value === offsets[0] - 1) { throw "selected failure"; } return value; }
                : (value: int): int => { return value + offsets[1]; };
            let caught = false;
            try { const mapped = values.map(failing); }
            catch (error) { strictEqual(String(error), "selected failure"); caught = true; }
            strictEqual(caught, true);
            let left = 0;
            let right = 0;
            {
                const observer = flag
                    ? (value: int): void => { left += value; }
                    : (value: int): void => { right += value; };
                const set = new Set<int>([1, 2]);
                set.forEach(observer);
                const map = new Map<int, int>([[1, 4], [2, 5]]);
                map.forEach(observer);
            }
            strictEqual(left, 12);
            strictEqual(right, 0);
        }
    "#);
}

#[test]
fn scoped_conditional_callbacks_reject_consuming_traversal_and_nullable_escape() {
    for (source, code) in [
        (r#"function main() {
            const values = [1]; const text = "owned";
            const selected = values.length === 1
                ? (value: int): string => { console.log(values.length); return text; }
                : (value: int): string => { console.log(values.length); return text; };
            const mapped = values.map(selected);
        }"#, "callback-access"),
        (r#"function escape() {
            const values = [1];
            const selected = values.length === 1
                ? (): int => values[0]
                : (): int => values.length;
            return values.length === 1 ? selected : null;
        }
        function main() {}"#, "callback-escape"),
        (r#"function main() {
            const values = [1];
            const selected = values.length === 1 ? (): int => values[0] : (): int => values.length;
            const optional = values.length === 1 ? selected : null;
            throw optional;
        }"#, "closure-escape"),
        (r#"function main() {
            const values = [1, 2];
            const selected = values.length === 2
                ? (value: int): int => { values.push(value); return value; }
                : (value: int): int => { values.push(value + 1); return value; };
            const mapped = values.map(selected);
        }"#, "capture-conflict"),
        (r#"function main() {
            const values = [1, 2];
            const selected = values.length === 2 ? (): int => values[0] : (): int => values[1];
            const optional = values.length === 2 ? selected : null;
            values[0] = 9;
            console.log(optional?.());
        }"#, "capture-conflict"),
    ] {
        let formatted = support::format_round_trip(source);
        let error = tsrust_compiler::check(&formatted).err().expect("must reject");
        assert_eq!(error.code, code, "{}", error.message);
    }
}

#[test]
fn scoped_nullable_carriers_cross_readonly_and_mutable_parameter_boundaries() {
    project(r#"
        import { strictEqual } from "std:testing";
        function read(action: (ReadOnly<Function<int, [value: int]>>) | null, value: int): int {
            return action?.(value) ?? -1;
        }
        function mutate(action: (Mutable<Function<int, [value: int]>>) | null, value: int): int {
            return action?.(value) ?? -1;
        }
        function main() {
            const offsets = [3, 7];
            const flag = offsets.length === 2;
            const selected = flag
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const inputs = [1, 2];
            const mapped = inputs.map(selected);
            strictEqual(mapped[1], 5);
            const optional = flag ? selected : null;
            strictEqual(read(optional, 2), 5);
            strictEqual(read(optional, 4), 7);
            let left = 0;
            let right = 0;
            {
                const selectedMutable = flag
                    ? (value: int): int => { left += value; return left; }
                    : (value: int): int => { right += value; return right; };
                const optionalMutable = flag ? selectedMutable : null;
                strictEqual(mutate(optionalMutable, 2), 2);
                strictEqual(mutate(optionalMutable, 4), 6);
            }
            strictEqual(left, 6);
            strictEqual(right, 0);
        }
    "#);
}
