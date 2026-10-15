import path from 'node:path';
import {root, temporary, rust, run} from './harness.mjs';
if (process.argv.length !== 3 || process.argv[2] !== 'demo') {
  console.error('Usage: node experiments/ui-state-lifecycle/cli.mjs demo');
  process.exitCode = 1;
} else {
  temporary(dir => {
    const binary = path.join(dir, 'demo');
    rust(path.join(root, 'lifecycle.rs'), binary);
    process.stdout.write(run(binary, []).stdout);
  });
}
