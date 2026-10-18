const input = new URL("http://localhost/?tag=one+two&tag=%C3%A9&bare&empty=&eq=a=b");
console.log(input.searchParams.getAll("tag").join("|"));
console.log(input.searchParams.get("missing") === null);
console.log(input.searchParams.get("bare")!.length);
console.log(input.searchParams.get("eq"));
