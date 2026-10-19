// Independent expected values from authored tokens, without the Tsrust style engine.
declare function require(name: string): {readFileSync(path: string, encoding: string): string};
declare const process: {argv: string[]};
declare const console: {log(value: string): void};
const theme = JSON.parse(require('node:fs').readFileSync(process.argv[2], 'utf8'));
function color(key: string): string {
  const seen = new Set<string>();
  while (true) {
    if (seen.has(key)) throw new Error('cycle');
    seen.add(key);
    const value: string = theme.dark[key];
    if (value.startsWith('@')) key = value.slice(1);
    else return value.toLowerCase();
  }
}
console.log(JSON.stringify({background: color('background'), border: color('border'), card: color('card'),
  foreground: color('foreground'), radius: theme.radius.lg, base: theme.typography.base,
  largePaddingX: theme.spacing.lg, largePaddingY: theme.spacing.md}));
