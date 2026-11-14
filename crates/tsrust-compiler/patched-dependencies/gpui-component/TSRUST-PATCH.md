# GPUI component 0.9.0 syntax capture patch

Source: published gpui-component0.9.0 archive SHA256
45bbfdba0efa392190f2fd446ec311325302eb047d5ce45dc1bcf189cb18f5eb.
Apache-2.0 source, license and upstream provenance are retained.

Preserve detailed theme capture names in SyntaxColors for both highlighter
backends; exact capture styles take priority and fall back through the nearest
parent. Syntect maps import/from/export keywords to keyword.import, parameters
to variable.parameter and quoted JSON keys to property.json_key using the enclosing
meta.mapping.key.json scope. JSON true/false literals map to boolean; null retains
its constant role. Escapes retain their own capture inside keys and values. This follows
Zed's capture/fallback model while retaining Syntect's existing language coverage.
Theme and lexical role regressions remain with the upstream highlighter tests.
The normalized Cargo manifest adds a standalone workspace for focused host tests.

Use compiler-managed patch payloads; registry bytes/checksums remain unchanged.
Remove the patch when an admitted upstream version preserves these theme names
and passes the role/fallback and native theme witnesses.

The published archive omits fixtures referenced by its test-only includes.
Those include paths now use local test-fixtures/ or the active sibling managed
GPUI base source. Test assertions stay unchanged. Four fixtures were fetched from
the exact .cargo_vcs_info.json commit7591a35eced761e3613c01e71ff04c10129dae1a
in https://github.com/longbridge/gpui-kit:

- crates/assets/assets/icons/arrow-up.svg: SHA256 8c22c25874c80bb062bef985d207d8f1d4a4d7847237d0a8cf720a3235968518
- crates/assets/assets/icons/search.svg: SHA256 283d371c2e433817bb9c0c8310caa6c77fa4177c0f4f1168d9c83b97af7389dc
- examples/fixtures/test.md: SHA256 415dafbd01b23ab58b45fbb57cc4bc3c8e4c7d5c568702e1763511abe9102097
- themes/aurora.json: SHA256 40176b7179352ade81371d7ebb27b1ad0144a0c506fd5637b5d8c3f58aee8b1f

The two icon fixtures retain the Lucide license in test-fixtures/LICENSE-LUCIDE.

Tab rendering now preserves an explicit text-size refinement instead of overwriting
it with the control-size default. Its inner and outer heights grow to fit that
text; default-size tabs retain their normal geometry. This lets editor and panel
labels follow Studio zoom through the shared renderer. Remove this patch when an
admitted upstream version preserves custom tab text sizes and passes native zoom
and clipping checks.

Sortable table headers now activate across the full header, including when column
selection is disabled. Clicking the sort icon stops propagation so it cycles once.
The cycle is ascending, descending, then original order, matching the public SDK
contract. A focused native TableState regression verifies the cycle without column
selection and preserves one active sort column. Remove when upstream
provides equivalent header activation and cycle behavior.

Button typography now inherits from its semantic root, so explicit font sizes and
interaction refinements reach labels without losing native variants, selection,
loading or focus behavior. Caller interaction styles merge after native colors
without registering duplicate GPUI hover/active styles. The renderer keeps native
presentation for typography/shrink utilities and retains the unstyled path for
authored surface/layout classes. A native font probe covers selected/unselected
buttons through hover transitions. Remove when upstream preserves equivalent
typography and interaction refinements.

Searchable dropdown rows now expose their item titles through the shared List
delegate, including custom-rendered rows. Selectable rows offer the same confirm
path to assistive technology as to pointer input. Disabled rows expose disabled
semantics and offer no press action; delegate confirmation keeps its existing
enabled-item guard. This repairs Select, ComboBox and MultiComboBox together.
Remove when upstream provides equivalent named, actionable and disabled rows.

Explicit control typography now reaches Select option rows and Pagination buttons.
Inputs and Selects grow their native frames and line heights to fit explicit font
sizes; native Buttons also size their icons and hit targets. TitleBar grows to fit
its controls and updates macOS traffic-light placement to the measured height.
A native geometry witness exercises 24px Button/Input/Select controls in a TitleBar.
The SDK forwards typography into these controls and uses a font-aware DataTable
row height without replacing stable sorting, column state or rich cells. Remove
when upstream preserves equivalent typography, measured sizing and zoom behavior.
