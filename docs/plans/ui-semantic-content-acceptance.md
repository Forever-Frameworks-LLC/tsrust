# Keyboard content and separator composition

The pinned Base UI + Nova Kbd/KbdGroup and Separator sources are ported as editable
offline components. Exact upstream blob identities and Nova token styles remain
canonical in SQLite. Public registry inspection is supplemental: its default
Radix result was rejected, and the explicitly selected Base UI registry was
checked against the pinned source. No GitHub account connection is required.

Both keyboard parts render semantic kbd containers, as in the pinned source.
Composed children retain ordinary checked ownership and live scalar bindings.
Keycaps display content without registering shortcuts. Separator is nonfocusable
and accessible, defaults horizontal and exposes data/ARIA orientation metadata.
Its vertical line follows the pinned self-stretch source rule; unrelated unused
Nova style definitions do not replace the actual source contract.

Shared support adds composed kbd/code View tags, inline-flex display, intrinsic
fit-content preferred sizes, text-xs typography, ancestor metadata predicates and
orientation selector aliases. Taffy's actual intrinsic Dimension support is used;
viewport widths are not substituted for missing named container queries. Theme
spacing, typography, alpha colors and caller class ordering remain shared across
targets. Native accessibility preserves descendants and separator orientation.

The portable props reject unsupported children/props explicitly. Arbitrary DOM
attributes, Base UI render/style callbacks, SVG children, Tooltip behavior and
named container queries remain pending. This section does not accept the full
Field family or the complete component inventory. Field still requires grouped
semantics, error lists, reactive metadata and responsive container behavior.

Focused witnesses are prepared across compiler/native behavior, editor help,
offline installation, a paired application and an independent Node-after-tsc
browser oracle. The standalone counter remains the user's visible working demo.
Acceptance results and consumer refresh follow below after the completed batch.
