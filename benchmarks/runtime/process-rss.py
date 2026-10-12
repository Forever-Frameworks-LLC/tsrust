"""One child per monitor: macOS ru_maxrss is bytes; no cumulative child reuse."""
import json
import resource
import subprocess
import sys

if sys.platform != 'darwin':
    raise SystemExit('RSS byte protocol currently requires macOS')
child = subprocess.run(sys.argv[1:], capture_output=True, text=True, timeout=60)
if child.returncode != 0:
    raise SystemExit(f'RSS child exited {child.returncode}: {child.stderr}')
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
if usage.ru_maxrss <= 0:
    raise SystemExit('Missing child peak RSS')
print(json.dumps({'stdout': child.stdout, 'stderr': child.stderr,
                  'peakRssBytes': usage.ru_maxrss}))
