# Desktop async and platform integration

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Scope UI-thread execution, component-owned tasks, system APIs, target checks, packaging, and assets.

**Planning deliverable:** A cancellation/cleanup contract, platform capability model, and staged packaging plan.

**Dependencies to coordinate:** [async-and-runtime](../guides/language/async-and-runtime.md), [desktop-reactivity](desktop-reactivity.md), [deployment](../guides/tooling/deployment.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-18"></a>

## Async Integration

The UI event loop and language async runtime must integrate cleanly.
Target:
function UserView(): View {
    const user = signal<User | null>(null);

    async function loadUser() {
        user = await api.getUser();
    }

    return (
        <Stack>
            <Button onClick={loadUser}>
                Load
            </Button>

            {user && <UserCard user={user} />}
        </Stack>
    );
}
Research:
* UI-thread restrictions
* background tasks
* cancellation
* thread-safe signals
* async event handlers
* synchronization back to UI thread
* structured concurrency tied to component lifetime

<a id="desktop-19"></a>

## Component-Scoped Async Lifetimes

Investigate whether async work started by a component can automatically belong to that component’s lifetime.
Example:
task(async () => {
    const user = await fetchUser();

    state.user = user;
});
If the component is destroyed before the operation completes, the task could potentially be cancelled automatically.
This should be evaluated as part of structured concurrency rather than implemented as hidden magic without clear semantics.

<a id="desktop-27"></a>

## Native System APIs

Eventually expose safe language-native APIs for:
clipboard
notifications
file dialogs
system tray
keyboard shortcuts
process management
filesystem
networking
camera
microphone
printing
screenshot APIs
window management
Platform-specific capabilities should be surfaced through explicit feature detection or compile-time target checks.

<a id="desktop-28"></a>

## Platform-Specific Code

Investigate syntax such as:
if platform.windows {
    ...
}
or compile-time constructs.
Alternatively:
import { Registry } from "windows";
should produce a compile-time error on unsupported targets unless guarded appropriately.
Do not make platform behavior mysterious.

<a id="desktop-29"></a>

## Native Packaging

Eventually investigate commands such as:
<cli> build --release
and:
<cli> package
Potential outputs:
Windows:
.exe
.msix
macOS:
.app
.dmg
Linux:
binary
AppImage
.deb
.rpm
Research:
* icons
* metadata
* code signing
* notarization
* installers
* application updates
* bundle resources

<a id="desktop-30"></a>

## Application Assets

Provide a standard resource mechanism.
Example structure:
src/
assets/
    icon.png
    fonts/
    images/
Potential API:
const icon = asset("./assets/icon.png");
The compiler can embed resources into the binary or package them appropriately.
