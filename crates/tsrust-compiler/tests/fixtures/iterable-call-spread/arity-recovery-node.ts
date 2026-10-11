// Node adaptation: explicit argument-count guards model the native checked contract.
import { strictEqual } from "std:testing";

type Counter = {
  value: int;
};

function fixed(counter: Mutable<Counter>, a: int, b: int): int {
  if (arguments.length !== 3) {
    throw {
      code: "call-arity"
    };
  }

  counter.value += 1;
  return a + b;
}

function plain(a: string, b: string): string {
  if (arguments.length !== 2) {
    throw {
      code: "call-arity"
    };
  }

  return a;
}

function main() {
  const counter: Counter = {
    value: 0
  };
  const short = new Set<int>([7]);
  let seen = 0;
  try {
    fixed(counter, ...short);
  } catch (error) {
    strictEqual(error.code, "call-arity");
    seen = 1;
  } finally {
    counter.value += 10;
  }

  strictEqual(seen, 1);
  strictEqual(counter.value, 10);
  strictEqual(short.size, 1);
  const one = String("x");
  try {
    plain(...one);
  } catch (error) {
    strictEqual(error.code, "call-arity");
    seen = 2;
  } finally {
    counter.value += 10;
  }

  strictEqual(seen, 2);
  strictEqual(counter.value, 20);
  strictEqual(one, "x");
  const items: int[] = [1];
  try {
    fixed(counter, ...items);
  } catch (error) {
    strictEqual(error.code, "call-arity");
    seen = 3;
  }

  strictEqual(seen, 3);
  strictEqual(counter.value, 20);
}
