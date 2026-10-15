const request = {pattern: '[0-9]+', text: 'Order 314 ready'};
const found = new RegExp(request.pattern).exec(request.text);
const description = found ? `Found ${found[0]} at byte ${Buffer.byteLength(request.text.slice(0, found.index))}` : 'No match';
console.log(description);
