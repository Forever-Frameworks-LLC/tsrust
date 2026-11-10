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
