# Assignment as a value

An assignment is an expression. Its value is the value that was assigned.

```tsrust
import { strictEqual } from "std:testing";

type Point = { x: int; label: string; };

function main() {
    let count = 0;
    let other = 0;
    count = other = 4;
    strictEqual(count + other, 8);

    let text = "a";
    strictEqual((text = "xyz").length, 3);
    strictEqual(text, "xyz");

    let point: Point = { x: 1, label: "p" };
    strictEqual((point = { x: 5, label: "q" }).x, 5);
    strictEqual((point.label = "longer").length, 6);

    let current = "";
    let rounds = 0;
    while ((current = rounds < 2 ? "go" : "") !== "") {
        rounds += 1;
    }
    strictEqual(rounds, 2);
}
```

For `int`, `number` and `boolean` the value is a copy, so assignments chain.

For a string, array or record the value is the target itself, read in place after
the store: `(text = "xyz").length` stores and then reads `text.length`. Nothing is
copied. The usual ownership rules follow from that:

- Reading a member, calling a method or passing the value to a read-only parameter
  leaves the target usable.
- Binding the value moves the target, exactly as naming the target would:
  after `const moved = (text = "new")`, `text` is no longer available. A chain
  `first = second = "same"` therefore leaves only `first` usable; assign each
  variable in its own statement, or use `clone()`.
- The target must be a `let` variable or a record field with a fixed path such as
  `holder.point.label`. For an array element or another computed place, the value of
  an owned assignment is not available, because the place would be evaluated twice;
  assign in a statement and read the place afterwards. Scalar elements work.
