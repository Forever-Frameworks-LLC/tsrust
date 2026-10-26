# windows-setup

Run `tsrust run` to execute the application. The development watcher (`tsrust dev`) is not yet supported on Windows.

- `tsrust run`: run once.
- `tsrust build`: build the configured binary.
- `tsrust check`: check project sources.
- `tsrust test`: run project tests.
- `tsrust format`: format project sources.

The nearest `tsrust.config.json` defines the project root, entry and output.
A `main` entrypoint is optional; this template includes one to print a greeting.
