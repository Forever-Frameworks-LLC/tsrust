# Empty components and paired typography

The six Empty exports from the pinned shadcn Base UI + Nova source pass focused
acceptance through `d72c8f6f`. The baseline is commit
`daaf156ee00ed91cc7e6bdd937c1b4973152d141`, Empty blob
`6a49a9813c02e2ce2548402e1a63b05de550bace`. The installed editable source keeps
`empty-icon`, default/icon media variants, source spacing and caller overrides.

Shared support includes dashed borders, tight tracking, relaxed line height,
nonnegative margins, balanced short text, theme-scaled maximum width, generic
heading families and bounded SVG/direct-Link descendant styles. Native text
measurement, painting and selection use the same settings. Native descendant
cascade now gives local child classes precedence over ancestor defaults, including
ancestor hover rules. Scoped theme ownership and quoted JSX ampersands are checked;
malformed numeric entities remain errors.

Four container slots preserve View replacement rendering. Title and Description
use TextProps for shared inline text/Link flow; they do not expose replacement
callbacks or arbitrary control children. React refs/events, unrestricted selectors,
negative/auto margins, named font files and identical platform text breaks remain
unsupported. Coverage remains partial rather than claiming complete DOM parity.

Focused evidence: seven native/model checks, generated-native installed component
and all four examples, exact ThemeTypography example, once-only resource release,
compiler compatibility boundaries, quoted-attribute/entity execution and rejection,
three completion rows in all four modes, receiver agreement, editor consistency,
installed-LSP protocol and the actual dev-browser Empty scenario. Browser checks
cover retained media/SVG identity, overrides, Create/reset, link hover, scoped fonts,
light/dark themes, accessibility and 390/768/1280 layouts. Existing native OS-window
startup evidence is reused; changed scene/layout behavior is tested directly.

SQLite stores `tsrust-empty-evidence.json`, original focused logs/references and
`tsrust-empty-closure.json`: 43 records, 200106 original bytes verified independently;
95351362 obsolete output bytes removed. No full compiler/LSP/SQL regression or
performance measurement is implied. Original failed runs remain preserved.

The documentation app reports 166 pages, 31 checked examples and 985 API declarations.
CLI includes `d72c8f6f`; editor API remains current at `2628a960` because the final
repair changes native cascade only. Preview startup latency and workspace-wide LSP
cost remain open investigations. Navigation/responsive repairs follow this section.
