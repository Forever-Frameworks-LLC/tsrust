import assert from 'node:assert/strict';
import {releaseDemo} from './release.mjs';
import path from 'node:path';
import {readFileSync} from 'node:fs';
import {root, temporary, execute, formattedWitnesses, reducedIdentityLimit} from './harness.mjs';
if (process.argv.length !== 3 || process.argv[2] !== 'demo') {
  console.error('Usage: node experiments/ui-tsrust-owner-handles/cli.mjs demo');
  process.exitCode = 1;
} else {
  temporary(dir => {
    formattedWitnesses(dir, assert);
    for (const file of ['identity.tsr', 'owners-lists.tsr', 'disposal.tsr', 'callback-lists.tsr', 'cycles.tsr', 'scope-exit.tsr', 'resource-capture.tsr', 'factory-self-replacement.tsr', 'named-self-replacement.tsr']) {
      process.stdout.write(execute(path.join(dir, file), dir));
    }
    reducedIdentityLimit(dir);
    process.stdout.write(execute(path.join(dir, 'identity-exhaustion.tsr'), dir));
    const release = releaseDemo();
    console.log(JSON.stringify({release}, null, 2));
    const matrix = JSON.parse(readFileSync(path.join(root, 'matrix.json'), 'utf8'));
    const followup = JSON.parse(readFileSync(path.join(root, 'resource-matrix.json'), 'utf8'));
    const identity = JSON.parse(readFileSync(path.join(root, 'identity-matrix.json'), 'utf8'));
    console.log(JSON.stringify({identity, historicalFollowup: followup, currentResourceContract: release.synchronousB2, historicalBlockers: matrix.blockers, historicalRows: matrix.rows}, null, 2));
    process.exitCode = 0;
  });
}
