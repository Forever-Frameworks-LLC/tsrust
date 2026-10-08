declare function require(name: string): {endianness():string};
console.log(require("node:os").endianness());
