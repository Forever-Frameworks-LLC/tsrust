# windows-setup

Run `tsrust dev` to rebuild and restart on source changes. Press Ctrl+C to stop.

- `tsrust run`: run once.
- `tsrust build`: build the configured binary.
- `tsrust check`: check project sources.
- `tsrust test`: run project tests.
- `tsrust format`: format project sources.

The nearest `tsrust.config.json` defines the project root, entry and output.
A `main` entrypoint is optional; this template includes one to print a greeting.
