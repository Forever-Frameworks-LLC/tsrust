# Styling and Themes

`className` takes Tailwind-style utilities: layout (`flex-row`, `gap-4`,
`grow`, `items-center`), sizing (`w-72`, `size-full`, `max-w-[960px]`),
spacing, borders, radius and typography.

## Colors

Theme colors such as `bg-primary`, `text-muted-foreground` and
`border-border` follow the active theme. The Tailwind palette works too
(`bg-sky-500`, `text-rose-600`), and so do arbitrary values such as
`bg-[#0ea5e9]` or `text-[oklch(0.7_0.15_150)]`.

```tsx live Swatches
function Swatches(): Node {
  return (
    <View className="flex-row gap-3">
      <View className="size-16 rounded-lg bg-primary" />
      <View className="size-16 rounded-lg bg-sky-500" />
      <View className="size-16 rounded-lg bg-amber-300" />
      <View className="size-16 rounded-lg bg-[#a855f7]" />
      <View className="size-16 rounded-lg border border-border bg-muted" />
    </View>
  );
}
```

## Light, dark and custom themes

`setTheme('dark')` switches GPUI Kit's dark theme for every window, and
`'system'` follows the OS. Register a theme set in GPUI Kit's JSON format and
apply one by name:

```tsx
import gruvbox from './themes/gruvbox.json';

function main() {
  registerThemes(gruvbox);
  useTheme('Gruvbox Light Medium');
  unwrap(run(<App />, 'App'));
}
```

This gallery uses that set; the title bar switch calls
`useTheme('Gruvbox Dark Medium')`.


## Exact and reactive values

Numeric spacing includes decimal steps (`p-13`, `gap-2.5`); theme spacing sets the
unit. Use exact values for dimensions and spacing (`w-[350px]`, `p-[13px]`,
`gap-[0.5rem]`), percentage dimensions (`w-[37%]`) or fractions (`w-2/5`). Values
must be finite and nonnegative; negative margins and offsets use a leading minus.

A class expression that reads a signal updates after a handler changes the signal:

```tsx live DynamicStyles
function DynamicStyles(): Node {
  const selected = signal(false);
  return (
    <View className="flex-col gap-3 w-full">
      <Button onPress={() => { selected.set(!selected.get()); }}>Toggle style</Button>
      <View className={selected.get() ? 'p-[13px] rounded-[10px] bg-primary text-primary-foreground w-2/5' : 'p-[13px] rounded-[10px] bg-muted w-[75%]'}>
        <Text>Live dimensions and colors</Text>
      </View>
      <Tag className={selected.get() ? 'p-4 bg-primary text-primary-foreground' : 'p-2 bg-muted'}>Kit classes update too</Tag>
      <Switch disabled={selected.get()} className="disabled:opacity-50">Disabled utility state</Switch>
      <View className="grid grid-cols-3 gap-[6px]">
        <Text className="italic underline">One</Text>
        <Text className="font-black line-through">Two</Text>
        <Text className="opacity-[0.37]">Three</Text>
      </View>
      <View className="hidden md:flex dark:bg-muted rounded-[10px] p-3"><Text>Visible from 768 logical pixels</Text></View>
    </View>
  );
}
```

## Inline text

Nested `Text` keeps font weight, italic style, generic font family, foreground,
solid background, underline and strikethrough in one wrapped paragraph. Dynamic classes
update those runs without changing the text value. A paragraph shares one font
size and line height; put differently sized text in separate `Text` elements.
Box layout and interaction utilities belong on the outer text element.
Place interactive `Link` components beside `Text` on desktop: links nested inside
a text paragraph do not yet have independent native pointer or keyboard targets.

```tsx live InlineTypography
function InlineTypography(): Node {
  const alternate = signal(false);
  return (
    <View className="flex-col gap-3">
      <Button onPress={() => { alternate.set(!alternate.get()); }}>Flip inline color</Button>
      <Text>Plain <Text as="span" className={alternate.get() ? 'font-bold text-sky-600' : 'font-bold text-rose-600'}>bold café</Text> and <Text as="span" className="italic font-mono bg-amber-200">inline code</Text>.</Text>
      <Text className="underline">Underlined <Text as="span" className="no-underline">plain span</Text> underlined.</Text>
      <Text className="font-mono">Monospace <Text as="span" className="font-sans">system sans</Text> monospace.</Text>
      <Button onPress={() => { alternate.set(!alternate.get()); }}><Text className="font-bold">Styled button label</Text></Button>
      <Text>Current color: {alternate.get() ? 'sky' : 'rose'}</Text>
    </View>
  );
}
```

GPUI currently leaves letter spacing, balanced wrapping, text transforms,
underline offsets, CSS animations, pointer-event/selection utilities and
content-sized fields, justify-self and auto-rows-min unchanged. Table CSS and nonuniform grid tracks use the
retained browser renderer; use native `Table`/`DataTable` and uniform grids on
desktop. Placeholder colors and inline interaction-state utilities are not mapped.
These are renderer limits even when a utility name is recognized by the shared
class parser.

Supported class families appear in completion and hover; unsupported utility names
fail checking. Arbitrary CSS expressions (`calc(...)`, arbitrary selectors and
unregistered CSS variables) are not a native styling API. GPUI has its own layout
and text engine, so browser-specific CSS needs an explicit native mapping.

Custom `Button` utilities own its appearance while GPUI Kit retains native focus,
Enter/Space activation, disabled behavior and accessibility roles. Use `aria-label`
when an icon or custom content does not supply a clear control name.
