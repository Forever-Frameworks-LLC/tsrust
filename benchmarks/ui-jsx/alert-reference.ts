// Independent CSS box-model oracle: a 256x192 border-box, 2px borders and 16px padding.
const border = 2;
const padding = 16;
const width = 256;
const height = 192;
const child = 32;
const content = width - 2 * (border + padding);
const geometry = [
  {index: 0, x: width - border - 8 - child, y: border + 8, width: child, height: child},
  {index: 1, x: border + 16, y: border + (height - 2 * border) / 2, width: width - 2 * border - 32, height: child},
  {index: 2, x: border + padding + content / 4, y: border + padding + 8, width: child, height: child},
  {index: 3, x: border - 8, y: border - 8, width: child, height: child},
];
console.log(JSON.stringify({geometry}));
