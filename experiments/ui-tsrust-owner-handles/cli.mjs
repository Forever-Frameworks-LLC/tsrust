import path from 'node:path';
import {readFileSync} from 'node:fs';
import {root, temporary, execute} from './harness.mjs';
if (process.argv.length !== 3 || process.argv[2] !== 'demo') {
  console.error('Usage: node experiments/ui-tsrust-owner-handles/cli.mjs demo');
  process.exitCode = 1;
} else {
  temporary(dir => {
    for (const file of ['owners-lists.tsr', 'disposal.tsr', 'callback-lists.tsr', 'cycles.tsr', 'domain-collision.tsr', 'scope-exit.tsr', 'runtime-boundary.tsr', 'resource-capture.tsr', 'factory-self-replacement.tsr', 'named-self-replacement.tsr']) {
      process.stdout.write(execute(path.join(root, 'witnesses', file), dir));
    }
    const matrix = JSON.parse(readFileSync(path.join(root, 'matrix.json'), 'utf8'));
    const followup = JSON.parse(readFileSync(path.join(root, 'resource-matrix.json'), 'utf8'));
    console.log(JSON.stringify({followup, investigation: 'complete', contract: 'blocked', blockers: matrix.blockers, rows: matrix.rows}, null, 2));
    process.exitCode = 2;
  });
}
