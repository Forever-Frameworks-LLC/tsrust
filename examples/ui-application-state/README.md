# Shared application contracts

Run `tsrust run` in this directory on macOS for a custom header that retains the
OS traffic-light controls. The header text and blank space drag the window;
the theme button remains interactive. Change `titlebar` to `native` in
`src/main.tsr` for the normal titlebar or other desktop platforms. This option
applies at window creation, not as a live toggle.

Increment the counter and edit Draft, switch light/dark mode, then toggle between
preview and selectable source. The counter and field values survive both actions.
The nested light scope remains independent. Hidden previews retain their state;
conditional JSX removal instead releases the removed subtree.

Both targets use the same checked source. The browser host owns its window chrome.
The repository browser builder can build this entry while integrated web build/dev
remains pending. `src/inspect.tsr` verifies initial document output without a GPU.
