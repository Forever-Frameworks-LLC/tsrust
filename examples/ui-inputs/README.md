# Editable controls

Run `tsrust dev` for the configured native target, or `tsrust build --target web`
for the browser. `src/main.tsr` is the configured entry; `src/app/page.tsrx` owns
application state and callbacks. The three component files are editable local
Base UI + Nova adaptations, also bundled for offline `tsrust ui add`.

`value` selects controlled text. Set application state from `onChange` to accept
an edit; otherwise the current value is restored. Omit `value` for an uncontrolled
field initialized by `defaultValue`. Direct signals and state-reading JSX value,
disabled and readOnly expressions refresh after handlers. Integer values remain
exact; editing always produces strings. CR/LF are removed from single-line input;
Textarea normalizes CRLF/CR to LF. Unpaired browser UTF-16 surrogates fail instead
of being replaced silently. Empty and zero controlled values take precedence.

Both targets keep composition preedit separate from committed application values.
The committed change invokes `onChange` once. The event contains read-only
`target` and `currentTarget` snapshots with value, name and id. It is not a DOM
Event: preventDefault, DOM node methods and composition callbacks are unsupported.
Native text editing uses Parley; clipboard access uses the operating system and
is released when the application exits. Clipboard failures keep the selection and
value unchanged and report a message. On Linux, retaining copied data after exit
requires a clipboard manager. Native platform autocomplete is not implemented;
autoComplete is a browser hint. Native accessibility currently exposes value,
label, description, focus, read-only/disabled/required/invalid state and SetValue;
platform accessibility text-range navigation remains pending.

This is the text-control slice, not complete Input parity: password, file, number,
date and other specialized input types fail explicitly. Arbitrary HTML/ARIA props,
refs, form submission, validation policy, resize handles and animated transitions
remain pending. Label peer/group disabled styling is also pending. Validation and
authentication policy stay in application code.
