// Independent CSS geometry oracle for the next paired layout component.
// Untimed; no compiler/runtime implementation is imported.
declare const console: { log(value: string): void };
interface Box {
  name: string;
  width: number;
  ratio: number;
  height?: number;
  minHeight?: number;
  maxHeight?: number;
}
const cases: Box[] = [
  { name: 'square', width: 320, ratio: 1 },
  { name: 'video', width: 320, ratio: 16 / 9 },
  { name: 'portrait', width: 120, ratio: 3 / 4 },
  { name: 'fractional-width', width: 317.5, ratio: 1.5 },
  { name: 'explicit-height', width: 320, ratio: 16 / 9, height: 128 },
  { name: 'minimum-height', width: 320, ratio: 16 / 9, minHeight: 256 },
  { name: 'maximum-height', width: 320, ratio: 1, maxHeight: 192 },
];
function dimensions(box: Box): { width: number; height: number } {
  if (!Number.isFinite(box.ratio) || box.ratio <= 0) throw new Error('A finite positive ratio is required');
  const height = box.height ?? box.width / box.ratio;
  return { width: box.width, height: Math.max(box.minHeight ?? 0, Math.min(box.maxHeight ?? Infinity, height)) };
}
if (dimensions(cases[0]).height !== 320 || dimensions(cases[1]).height !== 180) throw new Error('Invalid reference geometry');
console.log(JSON.stringify({ cases: cases.map(box => ({ name: box.name, ...dimensions(box) })) }));
