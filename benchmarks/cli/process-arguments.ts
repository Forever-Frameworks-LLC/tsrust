declare const process: { argv: string[] };
console.log(JSON.stringify(process.argv.slice(2)));
