// Independent notification oracle. Compile with tsc before running under Node.
declare const console: { log(value: string): void };

interface Snapshot {
  label: string;
  visible: boolean;
  notifications: number;
}

const observations: boolean[] = [false, true, true, false, true, false, true, true];
let previous: boolean | undefined;
let notifications = 0;
const steps: Snapshot[] = [];
for (const [index, visible] of observations.entries()) {
  // The final observation belongs to a fresh mount, so it has an initial event.
  if (index === observations.length - 1) previous = undefined;
  if (previous !== visible) {
    notifications++;
    previous = visible;
  }
  steps.push({
    label: `Visible: ${visible ? 'yes' : 'no'}; notifications: ${notifications}`,
    visible,
    notifications,
  });
}
console.log(JSON.stringify({ steps }));
