# Packaging and updating desktop apps

`tsrust package` turns a `std:ui` entry file into downloadable artifacts for the
host platform. The in-app updater in `std:ui` installs newer versions from the
GitHub releases of the repository named at packaging time.

## Start and run

```bash
tsrust new notes --ui
```

```bash
cd notes && tsrust dev
```

`new --ui` creates a desktop app from the GPUI starter. `tsrust dev` rebuilds and
restarts the window whenever a source or locale file changes. `tsrust ui list`
shows the GPUI Kit components; `tsrust ui add Settings DataTable` copies editable
wrappers into `src/components/` (`--all` adds every component, `--force`
replaces existing files).

## Package

```bash
tsrust package main.tsrx --name "Notes" --id com.example.notes --version 1.2.0 \
  --icon icon.png --repo owner/notes --out dist
```

| Option | Meaning |
| --- | --- |
| `--name` | Display and file name. Defaults to the entry file name, capitalized. |
| `--id` | Reverse-DNS bundle identifier. Defaults to `com.example.<name>`. |
| `--version` | `MAJOR.MINOR.PATCH`. Defaults to `0.1.0`. |
| `--icon` | A square PNG, ideally 1024×1024 with a transparent margin. macOS only. |
| `--repo` | GitHub `owner/name` whose releases provide updates. |
| `--out` | Output folder. Defaults to `dist`. |
| `--sign` | macOS code-signing identity, such as `Developer ID Application: Name (TEAMID)`. |
| `--notarize` | A `notarytool` keychain profile. Requires `--sign`. |

The command makes a release build, then writes per platform:

| Platform | Artifacts |
| --- | --- |
| macOS | `Name-1.2.0-macos-arm64.zip` (the `.app`, used by the updater) and `Name-1.2.0-macos-arm64.dmg` |
| Windows | `Name-1.2.0-windows-x64.zip` (the executable and `app.json`) |
| Linux | `Name-1.2.0-linux-x64.tar.gz`, and a `.deb` when `dpkg-deb` exists |

File names replace spaces in the name with dashes. Every artifact is listed in `SHA256SUMS`. `app.json` beside the executable
(`Contents/Resources` on macOS) records the name, identifier, version and
repository; `appInfo()` reads it.

Package on each operating system you ship. Each run only builds for its host.

## Sign and notarize on macOS

Unsigned apps open only after the user allows them in System Settings. For a
normal double-click launch, sign with a Developer ID certificate and notarize:

```bash
xcrun notarytool store-credentials notes-notary --apple-id you@example.com --team-id TEAMID
```

```bash
tsrust package main.tsrx --name Notes --version 1.2.0 --repo owner/notes \
  --sign "Developer ID Application: Your Name (TEAMID)" --notarize notes-notary
```

Signing uses the hardened runtime and a secure timestamp. Notarization submits
the zip, staples the ticket to the app, re-zips it, then signs, notarizes and
staples the DMG.

## Publish a release

Upload every artifact and `SHA256SUMS` to a GitHub release whose tag is the
version, with or without a leading `v`:

```bash
gh release create v1.2.0 dist/* --repo owner/notes --title "Notes 1.2.0"
```

The updater reads `releases/latest`, so drafts and prereleases are never offered.

## Update from the app

```tsrx
import { Button, appInfo, checkForUpdates, installUpdate, relaunch, notify } from 'std:ui';

<Button onPress={() => {
  checkForUpdates((update) => {
    if (update.available) {
      installUpdate((error) => { if (error === '') { relaunch(); } else { notify(error, { variant: 'error' }); } });
    }
  });
}}>Check for updates</Button>
```

- `checkForUpdates` compares the latest release with `appInfo().version`. A
  release is available only when it is newer and ships this platform's archive.
- `installUpdate` downloads the archive, checks its digest against the
  release's `SHA256SUMS`, and on macOS requires the same signing team as the
  installed app when that app is signed. It then swaps the new files in place.
- `relaunch` starts the installed app again and quits the current process.

Requests use the system `curl` (built into macOS, Windows 10+ and Linux
desktops) and the unauthenticated GitHub API, limited to 60 requests per hour
per address. Apps installed by a system package manager, such as the `.deb`,
cannot replace themselves and report an error; update them through that manager.
