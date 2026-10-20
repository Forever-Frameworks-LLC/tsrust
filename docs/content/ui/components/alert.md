<!-- tsrust-docs
{"id":"component-alert","group":"components","order":10,"status":"partial","targets":["web","native"],"component":"alert","examples":["alert-variants","alert-actions","alert-validation","alert-layouts","alert-theme"]}
-->
# Alert

Present contextual information without opening a dialog. Compose a title, description and optional application action. The default and destructive variants use your project's semantic tokens.

## Install

Run `tsrust ui add alert --offline`. Import `Alert`, `AlertTitle`, `AlertDescription` and `AlertAction` from `src/components/ui/alert`. The installed file is yours to edit. This version requires compiler API 3 for variant metadata and parent positioning; older compilers report that prerequisite instead of approximating the layout.

## Announcements and focus

`Alert` defaults to `role="alert"`, an assertive live region. Use it for information that needs immediate attention. Use `role="status"` for routine validation or saved-state messages that should be announced politely.

The container does not move focus. Buttons and links keep their ordinary keyboard behavior. An alert does not implement retry, dismissal, networking or authorization policy: the action example updates local state in its own handler. Hiding it retains its state and removes it from layout and accessibility.

`AlertTitle` styles content without choosing a heading level. Put a `Text` heading inside when the surrounding document needs one. Keep descriptions concise and associate form controls with visible labels.

## Action layout

Place `AlertAction` directly inside `Alert`. It is positioned at the top/right and reserves 18 spacing units for a compact action, 72 logical pixels with default tokens. Use a short control label.

Long or multiple actions belong in normal flow. The layout example overrides `absolute` with `relative`, clears the offsets with `top-auto right-auto`, and resets the container's reserved padding. Caller classes override each property independently.

## Variants and themes

`variant` defaults to `default`; `destructive` changes the foreground and description colors. `data-variant` mirrors that choice for checked descendant/group selectors. Metadata does not change accessibility by itself.

Use `card`, `cardForeground`, `destructive`, `mutedForeground` and `border` to customize the palette in `tsrust.config.json`. The scoped example changes its card tokens before construction and toggles its own mode without changing the documentation theme or remounting controls.

## Current boundaries

The four content/action subcomponents, inherited View metadata, retained hidden state and scoped mode switching work on web and native. Variant and ordinary custom class props are construction-time values; generic reactive custom props remain unfinished. SVG icons, arbitrary descendant CSS, automatic text balancing and Base UI render composition are not implemented. These are remaining clone capabilities, not silent fallbacks.

Each component consumes its forwarded children once. Application handlers keep the existing checked capture and cleanup rules; no public lifetime handle is needed.
