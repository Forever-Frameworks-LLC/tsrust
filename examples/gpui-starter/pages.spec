=== Button | button | Actions
A button with variants, sizes, icons and states. Buttons can be grouped, and a group can act as a toggle.
--- Basic Button
A labelled button that runs a handler when clicked.
function BasicButton(): Node {
  const clicks = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Button variant="primary" onPress={() => { clicks.set(clicks.get() + 1); }}>Click me</Button>
      <Text>Clicked {clicks} times</Text>
    </View>
  );
}
--- Variants
Without a variant a button uses GPUI Kit's neutral default style.
function ButtonVariants(): Node {
  return (
    <View className="flex-row flex-wrap gap-3">
      <Button variant="primary">Primary</Button>
      <Button>Default</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="danger">Delete</Button>
      <Button variant="warning">Warning</Button>
      <Button variant="success">Success</Button>
      <Button variant="info">Info</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="link">Link</Button>
      <Button variant="text">Text</Button>
    </View>
  );
}
--- Outline Buttons
Outline combines with any variant.
function OutlineButtons(): Node {
  return (
    <View className="flex-row gap-3">
      <Button variant="primary" outline={true}>Primary Outline</Button>
      <Button variant="danger" outline={true}>Danger Outline</Button>
    </View>
  );
}
--- Compact Button
Compact reduces the horizontal padding.
function CompactButton(): Node {
  return <Button compact={true}>Compact</Button>;
}
--- Sizes
xsmall, small, medium (the default) and large.
function ButtonSizes(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Button size="xsmall">Extra Small</Button>
      <Button size="small">Small</Button>
      <Button>Medium</Button>
      <Button size="large">Large</Button>
    </View>
  );
}
--- With Icons
A leading icon; a button with only an icon is square.
function IconButtons(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Button icon="check" variant="primary">Confirm</Button>
      <Button icon="heart">Like</Button>
      <Button icon="search" variant="ghost" />
    </View>
  );
}
--- With a dropdown caret icon
A caret after the label marks a button that opens a menu.
function CaretButton(): Node {
  return <Button caret={true}>Options</Button>;
}
--- Button States
Disabled, loading and selected. Loading shows a spinner and blocks clicks.
function ButtonStates(): Node {
  const saving = signal(false);
  return (
    <View className="flex-row items-center gap-3">
      <Button disabled={true}>Disabled</Button>
      <Button loading={saving.get()} onPress={() => { saving.set(!saving.get()); }}>Toggle loading</Button>
      <Button selected={true}>Selected</Button>
    </View>
  );
}
--- With Tooltip
A tooltip on hover. With tooltipCommand it also shows the keys of that Shortcut command.
function TooltipButton(): Node {
  const saved = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Shortcut command="save" keys="secondary-s" onTrigger={() => { saved.set(saved.get() + 1); }} />
      <Button tooltip="Save" tooltipCommand="save" onPress={() => { saved.set(saved.get() + 1); }}>Save</Button>
      <Text>Saved {saved} times</Text>
    </View>
  );
}
--- Spinner and Progress Icons
A spinner or a progress circle can take the icon's place.
function ProgressIcons(): Node {
  const progress = signal(45.0);
  return (
    <View className="flex-row items-center gap-3">
      <Button spinner={true}>Loading...</Button>
      <Button progress={progress.get()} onPress={() => { progress.set(progress.get() >= 100.0 ? 0.0 : progress.get() + 15.0); }}>Installing...</Button>
      <Button variant="primary" icon="save" loading={true}>Saving</Button>
    </View>
  );
}
--- Rounding
Corner rounding from none to large.
function ButtonRounding(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Button rounded="none">None</Button>
      <Button rounded="small">Small</Button>
      <Button>Medium</Button>
      <Button rounded="large">Large</Button>
    </View>
  );
}
--- Hover
onHover reports when the pointer enters and leaves.
function HoverButton(): Node {
  const hovered = signal(false);
  return (
    <View className="flex-row items-center gap-3">
      <Button onHover={(on) => { hovered.set(on); }}>Hover me</Button>
      <Text>{hovered.get() ? 'Hovering' : 'Not hovering'}</Text>
    </View>
  );
}
--- Button Group
Buttons joined into one control; group options apply to every button.
function GroupOfButtons(): Node {
  const last = signal('none');
  return (
    <View className="flex-col gap-3">
      <ButtonGroup variant="primary">
        <Button onPress={() => { last.set('one'); }}>One</Button>
        <Button onPress={() => { last.set('two'); }}>Two</Button>
        <Button icon="ellipsis" onPress={() => { last.set('more'); }} />
      </ButtonGroup>
      <Text>Last clicked: {last}</Text>
    </View>
  );
}
--- Toggle Button Group
With multiple, clicks toggle each button; onClick reports the selected indexes.
function ToggleButtonGroup(): Node {
  const selected = signal('0');
  return (
    <View className="flex-col gap-3">
      <ButtonGroup outline={true} multiple={true} onClick={(indexes) => { selected.set(clone(indexes)); }}>
        <GroupButton selected={selected.get().includes('0')}>Option 1</GroupButton>
        <GroupButton selected={selected.get().includes('1')}>Option 2</GroupButton>
        <GroupButton selected={selected.get().includes('2')}>Option 3</GroupButton>
      </ButtonGroup>
      <Text>Selected: {selected}</Text>
    </View>
  );
}
--- Vertical Group
Groups can stack vertically and take any size.
function VerticalGroup(): Node {
  return (
    <ButtonGroup vertical={true} size="small" compact={true}>
      <Button icon="arrow-up">Up</Button>
      <Button icon="arrow-down">Down</Button>
    </ButtonGroup>
  );
}
--- Custom Variant
A custom fill, text, hover and pressed color, from theme color names or hex.
function CustomVariant(): Node {
  return <Button color="magenta" foreground="primary-foreground" hoverColor="magenta-light" activeColor="magenta">Custom Button</Button>;
}
=== Toggle | toggle | Actions
A button that stays pressed while on. Toggles can be grouped, optionally as one segmented control.
--- Basic Toggle
A toggle with text content.
function BasicToggle(): Node {
  const on = signal(false);
  return <Toggle checked={on.get()} onChange={(checked) => { on.set(checked); }}>Bold</Toggle>;
}
--- Icon Toggle
A toggle can show an icon instead of text.
function IconToggle(): Node {
  const on = signal(true);
  return <Toggle icon="eye" tooltip="Show preview" checked={on.get()} onChange={(checked) => { on.set(checked); }} />;
}
--- Sizes
xsmall, small, medium and large.
function ToggleSizes(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Toggle size="xsmall">XS</Toggle>
      <Toggle size="small">Small</Toggle>
      <Toggle>Medium</Toggle>
      <Toggle size="large">Large</Toggle>
    </View>
  );
}
--- Disabled State
A disabled toggle keeps its state but cannot change.
function DisabledToggle(): Node {
  return (
    <View className="flex-row gap-3">
      <Toggle disabled={true}>Off</Toggle>
      <Toggle checked={true} disabled={true}>On</Toggle>
    </View>
  );
}
--- Toggle Group
Toggles in a row; each reports its own change.
function TextFormatting(): Node {
  const bold = signal(true);
  const italic = signal(false);
  const underline = signal(false);
  return (
    <ToggleGroup>
      <Toggle icon="bold" checked={bold.get()} onChange={(on) => { bold.set(on); }} />
      <Toggle icon="italic" checked={italic.get()} onChange={(on) => { italic.set(on); }} />
      <Toggle icon="underline" checked={underline.get()} onChange={(on) => { underline.set(on); }} />
    </ToggleGroup>
  );
}
--- Segmented Group
A segmented group draws as one control, in any size.
function SegmentedGroup(): Node {
  const view = signal(0);
  return (
    <View className="flex-row gap-6">
      <ToggleGroup segmented={true}>
        <Toggle checked={view.get() === 0} onChange={(on) => { if (on) { view.set(0); } }}>List</Toggle>
        <Toggle checked={view.get() === 1} onChange={(on) => { if (on) { view.set(1); } }}>Grid</Toggle>
      </ToggleGroup>
      <ToggleGroup segmented={true} size="small">
        <Toggle>Top</Toggle>
        <Toggle>Bottom</Toggle>
      </ToggleGroup>
    </View>
  );
}
=== Shortcut | shortcut | Actions
Binds keystrokes to a named command while it is mounted. Tooltips and menus that name the command show its keys.
--- Basic Shortcut
Press the keys anywhere in the window. secondary means cmd on macOS and ctrl elsewhere.
function BasicShortcut(): Node {
  const count = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Shortcut command="increment" keys="secondary-k" onTrigger={() => { count.set(count.get() + 1); }} />
      <Kbd>secondary-k</Kbd>
      <Text>Pressed {count} times</Text>
    </View>
  );
}
=== Menu | menu | Overlays
Dropdown and context menus with icons, check marks, labels, separators, links, custom rows, keyboard shortcuts and nested submenus.
--- Dropdown Menu
A button that opens a menu.
function BasicDropdown(): Node {
  const last = signal('none');
  return (
    <View className="flex-row items-center gap-3">
      <DropdownMenu label="Actions">
        <MenuItem onSelect={() => { last.set('copy'); }}>Copy</MenuItem>
        <MenuItem onSelect={() => { last.set('paste'); }}>Paste</MenuItem>
        <MenuItem onSelect={() => { last.set('delete'); }}>Delete</MenuItem>
      </DropdownMenu>
      <Text>Last action: {last}</Text>
    </View>
  );
}
--- Context Menu
Right-click the area to open its menu.
function BasicContextMenu(): Node {
  const last = signal('none');
  return (
    <ContextMenu>
      <View className="rounded-lg border border-border p-8"><Text>Right-click here (last: {last})</Text></View>
      <MenuItem onSelect={() => { last.set('rename'); }}>Rename</MenuItem>
      <MenuItem onSelect={() => { last.set('duplicate'); }}>Duplicate</MenuItem>
    </ContextMenu>
  );
}
--- Split Button
With onClick the button runs its own action and only the arrow opens the menu.
function SplitDropdown(): Node {
  const runs = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <DropdownMenu label="Run" variant="primary" onClick={() => { runs.set(runs.get() + 1); }}>
        <MenuItem>Run tests</MenuItem>
        <MenuItem>Run benchmarks</MenuItem>
      </DropdownMenu>
      <Text>Ran {runs} times</Text>
    </View>
  );
}
--- Custom Trigger and Anchor
A Button child is the trigger with all its options; anchor picks the menu corner aligned to it.
function TriggerAndAnchor(): Node {
  return (
    <DropdownMenu anchor="top-right" menuLabel="More actions">
      <Button icon="ellipsis" variant="ghost" />
      <MenuItem icon="settings">Settings</MenuItem>
      <MenuItem icon="log-out">Sign out</MenuItem>
    </DropdownMenu>
  );
}
--- Icons, Disabled and Check State
Entries can carry an icon, be disabled or show a check mark on either side.
function EntryStates(): Node {
  const wrap = signal(true);
  return (
    <View className="flex-row gap-3">
      <DropdownMenu label="View">
        <MenuItem checked={wrap.get()} onSelect={() => { wrap.set(!wrap.get()); }}>Word wrap</MenuItem>
        <MenuItem icon="map" disabled={true}>Minimap</MenuItem>
      </DropdownMenu>
      <DropdownMenu label="Checks on the right" checkSide="right">
        <MenuItem checked={wrap.get()} onSelect={() => { wrap.set(!wrap.get()); }}>Word wrap</MenuItem>
      </DropdownMenu>
    </View>
  );
}
--- Labels, Separators and Links
Labels head groups of entries; separators divide them; a link entry opens a URL.
function LabelsAndLinks(): Node {
  return (
    <DropdownMenu label="Help">
      <MenuLabel>Documentation</MenuLabel>
      <MenuItem href="https://gpui-kit.com">GPUI Kit</MenuItem>
      <MenuItem href="https://github.com/Forever-Frameworks-LLC/tsrust">Tsrust</MenuItem>
      <MenuSeparator />
      <MenuItem icon="info">About</MenuItem>
    </DropdownMenu>
  );
}
--- Custom Element
Text children are the item label; other children draw a custom row.
function CustomEntry(): Node {
  return (
    <DropdownMenu label="Account">
      <MenuItem><View className="flex-col"><Text className="font-semibold">Ada Lovelace</Text><Text className="text-muted-foreground">ada@example.com</Text></View></MenuItem>
      <MenuSeparator />
      <MenuItem icon="user">Profile</MenuItem>
    </DropdownMenu>
  );
}
--- Keyboard Shortcuts
A MenuItem naming a Shortcut command shows its keys and runs it.
function ShortcutEntries(): Node {
  const saved = signal(0);
  return (
    <View className="flex-row items-center gap-3">
      <Shortcut command="menu-save" keys="secondary-shift-s" onTrigger={() => { saved.set(saved.get() + 1); }} />
      <DropdownMenu label="File">
        <MenuItem command="menu-save">Save all</MenuItem>
      </DropdownMenu>
      <Text>Saved {saved} times</Text>
    </View>
  );
}
--- Submenus
Submenus nest, with icons.
function NestedMenus(): Node {
  return (
    <DropdownMenu label="Share">
      <MenuItem icon="link">Copy link</MenuItem>
      <Submenu label="Send to" icon="send">
        <MenuItem>Email</MenuItem>
        <Submenu label="Chat">
          <MenuItem>Team</MenuItem>
          <MenuItem>Direct message</MenuItem>
        </Submenu>
      </Submenu>
    </DropdownMenu>
  );
}
--- Scrollable Menus and Sizing
Long menus scroll within a maximum height; widths can be bounded.
function LongMenu(): Node {
  return (
    <DropdownMenu label="Font" scrollable={true} maxHeight={200} minWidth={180}>
      <MenuItem>Inter</MenuItem>
      <MenuItem>Roboto</MenuItem>
      <MenuItem>Source Sans</MenuItem>
      <MenuItem>Fira Sans</MenuItem>
      <MenuItem>IBM Plex Sans</MenuItem>
      <MenuItem>Noto Sans</MenuItem>
      <MenuItem>Open Sans</MenuItem>
      <MenuItem>Lato</MenuItem>
    </DropdownMenu>
  );
}
=== Badge | badge | Data Display
Counts, dots or icons on another element, for notifications and status.
--- Badge with Count
A count above zero shows the badge; above max (default 99) it reads 99+.
function BadgeCount(): Node {
  return <Badge count={3}><Icon name="bell" size="lg" /></Badge>;
}
--- Variants
A count, a dot or an icon.
function BadgeVariants(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Badge count={5}><Avatar name="Ada Lovelace" /></Badge>
      <Badge dot={true}><Icon name="inbox" size="lg" /></Badge>
      <Badge icon="check"><Avatar name="Alan Turing" /></Badge>
    </View>
  );
}
--- Badge Sizes
small, medium and large.
function BadgeSizes(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Badge size="small" count={1}><Avatar name="Ada" size="small" /></Badge>
      <Badge count={5}><Avatar name="Grace" /></Badge>
      <Badge size="large" count={10}><Avatar name="Linus" size="large" /></Badge>
    </View>
  );
}
--- Badge Colors
A theme color, a palette color such as violet-500, or hex.
function BadgeColors(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Badge count={3} color="blue"><Avatar name="Ada" /></Badge>
      <Badge icon="star" color="yellow"><Avatar name="Grace" /></Badge>
      <Badge dot={true} color="violet-500"><Icon name="bell" size="lg" /></Badge>
    </View>
  );
}
--- Badge on Icons
High counts cap at max.
function BadgeOnIcons(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Badge count={3}><Icon name="bell" size="lg" /></Badge>
      <Badge count={103}><Icon name="inbox" size="lg" /></Badge>
      <Badge count={150} max={999}><Icon name="mail" size="lg" /></Badge>
    </View>
  );
}
--- Complex Nested Badges
Badges nest for combined status.
function NestedBadges(): Node {
  return (
    <View className="flex-row items-center gap-8">
      <Badge count={212} size="large">
        <Badge icon="check" size="large" color="cyan"><Avatar name="Ada Lovelace" size="large" /></Badge>
      </Badge>
      <Badge count={2} color="green" size="large">
        <Badge icon="star" size="large" color="yellow"><Avatar name="Grace Hopper" size="large" /></Badge>
      </Badge>
    </View>
  );
}
=== Tag | tag | Data Display
Compact labels for categories, status and metadata.
--- Basic Tags
Semantic variants.
function BasicTags(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Tag variant="primary">Primary</Tag>
      <Tag variant="secondary">Secondary</Tag>
      <Tag variant="danger">Danger</Tag>
      <Tag variant="success">Success</Tag>
      <Tag variant="warning">Warning</Tag>
      <Tag variant="info">Info</Tag>
    </View>
  );
}
--- Outline Tags
Outline draws only the border and text.
function OutlineTags(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Tag variant="primary" outline={true}>Primary Outline</Tag>
      <Tag variant="secondary" outline={true}>Secondary Outline</Tag>
      <Tag variant="danger" outline={true}>Error Outline</Tag>
      <Tag variant="success" outline={true}>Success Outline</Tag>
      <Tag variant="warning" outline={true}>Warning Outline</Tag>
      <Tag variant="info" outline={true}>Info Outline</Tag>
    </View>
  );
}
--- Tag Sizes
small and medium.
function TagSizes(): Node {
  return (
    <View className="flex-row items-center gap-2">
      <Tag variant="primary" size="small">Small Tag</Tag>
      <Tag variant="primary">Medium Tag</Tag>
    </View>
  );
}
--- Custom Colors
Named color families.
function PaletteTags(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Tag palette="blue">Blue Tag</Tag>
      <Tag palette="green">Green Tag</Tag>
      <Tag palette="purple">Purple Tag</Tag>
      <Tag palette="pink">Pink Tag</Tag>
      <Tag palette="indigo">Indigo Tag</Tag>
      <Tag palette="yellow">Yellow Tag</Tag>
      <Tag palette="red">Red Tag</Tag>
    </View>
  );
}
--- Custom Fill Colors
Exact fill, text and border colors: hex or palette names such as sky-100.
function CustomTag(): Node {
  return (
    <View className="flex-row gap-2">
      <Tag color="#1f6feb" foreground="#ffffff" borderColor="#1a5fc8">Custom Color</Tag>
      <Tag color="sky-100" foreground="sky-800" borderColor="sky-300">Palette Color</Tag>
    </View>
  );
}
--- Rounded Corners
Pill shape or a radius in pixels.
function RoundedTags(): Node {
  return (
    <View className="flex-row gap-2">
      <Tag variant="primary" roundedFull={true}>Rounded Full</Tag>
      <Tag variant="primary" rounded={4}>Custom Radius</Tag>
      <Tag variant="primary" rounded={0}>Square Tag</Tag>
    </View>
  );
}
--- Combined Styles
Options combine.
function CombinedTags(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Tag variant="primary" size="small" roundedFull={true}>Small Pill</Tag>
      <Tag variant="success" size="small" roundedFull={true}>Success Pill</Tag>
      <Tag variant="warning" outline={true} rounded={2}>Custom Outline</Tag>
      <Tag palette="purple" outline={true}>Purple Outline</Tag>
    </View>
  );
}
=== Label | label | Data Display
Text with optional secondary text, highlighting and masking.
--- Basic Label
Text content is the label.
function BasicLabel(): Node {
  return <Label>This is a label</Label>;
}
--- Label with Secondary Text
A Text child with variant="secondary" fills the muted secondary text.
function SecondaryLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label>
        Company Address
        <Text variant="secondary">(optional)</Text>
      </Label>
      <Label>
        <Text variant="inherit">Email Address</Text>
        <Text variant="secondary">(required)</Text>
      </Label>
    </View>
  );
}
--- Text Alignment
Alignment utilities on a full-width label.
function AlignedLabels(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Label className="w-full">Text align left</Label>
      <Label className="w-full text-center">Text align center</Label>
      <Label className="w-full text-right">Text align right</Label>
    </View>
  );
}
--- Text Highlighting
Highlights every match, or only a match at the start.
function HighlightedLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label highlight="Hello">Hello World Hello</Label>
      <Label highlight="Hello" highlightPrefix={true}>Hello World</Label>
      <Label highlight="Company">
        Company Name
        <Text variant="secondary">(optional)</Text>
      </Label>
    </View>
  );
}
--- Color and Styling
Text utilities style the label.
function StyledLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label className="text-primary">Color Label</Label>
      <Label className="text-rose-500">Palette Color Label</Label>
      <Label className="text-xl font-semibold">Font Size Label</Label>
    </View>
  );
}
--- Masked Labels
Masked text draws as bullets.
function MaskedLabel(): Node {
  const masked = signal(true);
  return (
    <View className="flex-row items-center gap-3">
      <Label masked={masked.get()} className="text-2xl">9,182.1 USD</Label>
      <Button size="small" onPress={() => { masked.set(!masked.get()); }}>{masked.get() ? 'Show' : 'Hide'}</Button>
    </View>
  );
}
--- Multi-line Text
Labels wrap inside a narrow width.
function WrappedLabel(): Node {
  return <Label className="w-[200px]">Label should support text wrap in default, if the text is too long, it should wrap to the next line.</Label>;
}
--- Different Sizes
Text size utilities.
function LabelSizes(): Node {
  return (
    <View className="flex-col gap-1">
      <Label className="text-2xl">Extra Large</Label>
      <Label className="text-xl">Large</Label>
      <Label>Medium</Label>
      <Label className="text-sm">Small</Label>
      <Label className="text-xs">Extra Small</Label>
    </View>
  );
}
=== Kbd | kbd | Data Display
Keyboard shortcuts formatted for the current platform.
--- Common Shortcuts
Keys use GPUI keystroke syntax; secondary is cmd on macOS and ctrl elsewhere.
function CommonShortcuts(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Kbd>cmd-shift-p</Kbd>
      <Kbd>cmd-t</Kbd>
      <Kbd>escape</Kbd>
      <Kbd>enter</Kbd>
      <Kbd>backspace</Kbd>
    </View>
  );
}
--- Multiple Modifiers
Complex combinations.
function ModifierKeys(): Node {
  return (
    <View className="flex-row gap-2">
      <Kbd>cmd-ctrl-shift-a</Kbd>
      <Kbd>cmd-alt-backspace</Kbd>
      <Kbd>ctrl-alt-shift-a</Kbd>
    </View>
  );
}
--- Arrow Keys and Function Keys
Named keys.
function NamedKeys(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Kbd>left</Kbd>
      <Kbd>right</Kbd>
      <Kbd>up</Kbd>
      <Kbd>down</Kbd>
      <Kbd>f12</Kbd>
      <Kbd>secondary-f12</Kbd>
      <Kbd>pageup</Kbd>
      <Kbd>pagedown</Kbd>
    </View>
  );
}
--- Without Visual Styling
Plain draws only the key text.
function PlainKbd(): Node {
  return <Kbd plain={true}>cmd-s</Kbd>;
}
--- From Command Bindings
Shows the keys bound to a Shortcut command.
function CommandKbd(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Shortcut command="open-palette" keys="secondary-shift-o" />
      <Text>Open palette:</Text>
      <Kbd command="open-palette" />
    </View>
  );
}
--- Keyboard Shortcut Help
Shortcuts beside descriptions.
function ShortcutHelp(): Node {
  return (
    <View className="flex-col gap-2">
      <View className="flex-row items-center gap-2"><Text>Open command palette:</Text><Kbd>cmd-shift-p</Kbd></View>
      <View className="flex-row items-center gap-2"><Text>Save file:</Text><Kbd>cmd-s</Kbd></View>
      <View className="flex-row items-center gap-2"><Text>Find in files:</Text><Kbd>cmd-shift-f</Kbd></View>
    </View>
  );
}
--- Inline Documentation
Keys inside a sentence.
function InlineKbd(): Node {
  return (
    <View className="flex-row items-center gap-1">
      <Text>Press</Text><Kbd>escape</Kbd><Text>to cancel or</Text><Kbd>enter</Kbd><Text>to confirm.</Text>
    </View>
  );
}
--- Custom Styling
Utility classes restyle the key cap.
function StyledKbd(): Node {
  return <Kbd className="text-primary border-primary bg-primary/10">cmd-k</Kbd>;
}
=== Skeleton | skeleton | Feedback
Animated placeholders while content loads.
--- Basic Skeleton
A full-width line by default.
function BasicSkeleton(): Node {
  return <Skeleton className="w-80" />;
}
--- Text Line Skeleton
Lines of varying width.
function TextSkeleton(): Node {
  return (
    <View className="flex-col gap-2">
      <Skeleton className="w-[250px] h-4 rounded" />
      <Skeleton className="w-[200px] h-4 rounded" />
      <Skeleton className="w-[180px] h-4 rounded" />
    </View>
  );
}
--- Circle Skeleton
Avatar placeholders.
function CircleSkeleton(): Node {
  return (
    <View className="flex-row gap-3">
      <Skeleton className="size-12 rounded-full" />
      <Skeleton className="size-16 rounded-full" />
    </View>
  );
}
--- Different Shapes
Sizes and radii come from utilities.
function SkeletonShapes(): Node {
  return (
    <View className="flex-col gap-3 w-96">
      <Skeleton className="w-[200px] h-4 rounded-sm" />
      <Skeleton className="size-20 rounded" />
      <Skeleton className="w-full h-[120px] rounded-lg" />
      <Skeleton className="size-6 rounded" />
    </View>
  );
}
--- Secondary Variant
A subtler fill.
function SecondarySkeleton(): Node {
  return <Skeleton secondary={true} className="w-80" />;
}
--- Loading Profile Card
Shapes compose into a layout.
function ProfileSkeleton(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Skeleton className="size-12 rounded-full" />
      <View className="flex-col gap-2">
        <Skeleton className="w-[150px] h-4" />
        <Skeleton className="w-[100px] h-3" />
      </View>
    </View>
  );
}
--- Conditional Loading
Swap the skeleton for content when loaded.
function ConditionalSkeleton(): Node {
  const loading = signal(true);
  return (
    <View className="flex-col gap-3 w-80">
      <Switch checked={loading.get()} onChange={(on) => { loading.set(on); }}>Loading</Switch>
      {loading.get() ? <Skeleton className="w-full h-4" /> : <Text>Content loaded</Text>}
    </View>
  );
}
=== Spinner | spinner | Feedback
A spinning icon for indeterminate loading.
--- Basic
The default loader icon.
function BasicSpinner(): Node {
  return <Spinner />;
}
--- Spinner with Custom Color
A theme color, a palette color such as blue-500, or hex.
function ColoredSpinners(): Node {
  return (
    <View className="flex-row gap-4">
      <Spinner color="primary" />
      <Spinner color="blue-500" />
      <Spinner color="emerald-500" />
      <Spinner color="#e8590c" />
    </View>
  );
}
--- Spinner Sizes
xsmall to large.
function SpinnerSizes(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Spinner size="xsmall" />
      <Spinner size="small" />
      <Spinner />
      <Spinner size="large" />
    </View>
  );
}
--- Spinner with Custom Icon
Any icon can spin.
function IconSpinners(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Spinner icon="loader-circle" />
      <Spinner icon="loader-circle" size="large" color="blue" />
      <Spinner icon="refresh-cw" />
    </View>
  );
}
--- Easing
The rotation easing curve.
function SpinnerEasing(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Spinner easing="linear" />
      <Spinner easing="ease-in-out" />
      <Spinner easing="quadratic" />
      <Spinner easing="ease-out-quint" />
    </View>
  );
}
--- In UI Components
Inside a button or beside text.
function SpinnerInContext(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Button loading={true}>Submit</Button>
      <View className="flex-row items-center gap-2"><Spinner size="small" /><Text>Loading...</Text></View>
    </View>
  );
}
=== Progress | progress | Feedback
Bar and circle indicators for determinate or loading progress.
--- Different Progress Values
Values from 0 to 100.
function ProgressValues(): Node {
  return (
    <View className="flex-col gap-3 w-80">
      <Progress value={0} />
      <Progress value={25} />
      <Progress value={75} />
      <Progress value={100} />
    </View>
  );
}
--- Loading State
An animated bar before the value is known.
function LoadingProgress(): Node {
  return <Progress loading={true} className="w-80" />;
}
--- Sizes
xsmall to large heights.
function ProgressSizes(): Node {
  return (
    <View className="flex-col gap-3 w-80">
      <Progress value={50} size="xsmall" />
      <Progress value={50} size="small" />
      <Progress value={50} />
      <Progress value={50} size="large" />
    </View>
  );
}
--- Custom Style
Color and utility classes.
function StyledProgress(): Node {
  return <Progress value={60} color="green" className="w-80 h-2 rounded-none" />;
}
--- Dynamic Progress Updates
Live values from state.
function DynamicProgress(): Node {
  const value = signal<number>(30);
  return (
    <View className="flex-col gap-3 w-80">
      <Progress value={value.get()} accessibilityLabel="Download" />
      <View className="flex-row gap-2">
        <Button size="small" onPress={() => { value.set(Math.max(0, value.get() - 10)); }}>-10</Button>
        <Button size="small" onPress={() => { value.set(Math.min(100, value.get() + 10)); }}>+10</Button>
      </View>
    </View>
  );
}
--- Progress Circle
A circular indicator, with the same values, loading state and sizes.
function CircleProgress(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <ProgressCircle value={25} size="small" />
      <ProgressCircle value={50} />
      <ProgressCircle value={75} size="large" color="green" />
      <ProgressCircle loading={true} />
    </View>
  );
}
--- With Inner Content
Children draw inside the circle.
function CircleContent(): Node {
  return (
    <ProgressCircle value={68} className="size-16">
      <Text className="text-xs">68%</Text>
    </ProgressCircle>
  );
}
--- Inline with Label
A small circle beside text.
function InlineCircle(): Node {
  return (
    <View className="flex-row items-center gap-2">
      <ProgressCircle value={40} size="small" />
      <Text>Uploading 2 of 5 files</Text>
    </View>
  );
}
=== Avatar | avatar | Data Display
User images with initials and icon fallbacks, alone or grouped.
--- Basic Avatar
An image source.
function BasicAvatar(): Node {
  return <Avatar name="Jason Lee" src="https://avatars.githubusercontent.com/u/5518?v=4" />;
}
--- Avatar with Fallback Text
Initials from the name when there is no image.
function InitialsAvatar(): Node {
  return (
    <View className="flex-row gap-3">
      <Avatar name="John Doe" />
      <Avatar name="Ada Lovelace" />
    </View>
  );
}
--- Avatar Placeholder
An icon when there is no image or name.
function PlaceholderAvatar(): Node {
  return (
    <View className="flex-row gap-3">
      <Avatar />
      <Avatar placeholder="building-2" />
    </View>
  );
}
--- Avatar Sizes
xsmall to large.
function AvatarSizes(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Avatar name="Ada" size="xsmall" />
      <Avatar name="Ada" size="small" />
      <Avatar name="Ada" />
      <Avatar name="Ada" size="large" />
    </View>
  );
}
--- Custom Styling
Utility classes such as size and border.
function StyledAvatar(): Node {
  return <Avatar name="Grace Hopper" className="size-16 border border-primary" />;
}
--- Avatar Group
Overlapping avatars.
function BasicAvatarGroup(): Node {
  return (
    <AvatarGroup>
      <Avatar name="Ada Lovelace" />
      <Avatar name="Alan Turing" />
      <Avatar name="Grace Hopper" />
    </AvatarGroup>
  );
}
--- Group with Limit
Avatars past the limit are hidden, or summarized by an ellipsis.
function LimitedAvatarGroup(): Node {
  return (
    <View className="flex-col items-start gap-3">
      <AvatarGroup limit={3}>
        <Avatar name="Ada Lovelace" />
        <Avatar name="Alan Turing" />
        <Avatar name="Grace Hopper" />
        <Avatar name="Linus Torvalds" />
        <Avatar name="Margaret Hamilton" />
      </AvatarGroup>
      <AvatarGroup limit={3} ellipsis={true}>
        <Avatar name="Ada Lovelace" />
        <Avatar name="Alan Turing" />
        <Avatar name="Grace Hopper" />
        <Avatar name="Linus Torvalds" />
      </AvatarGroup>
    </View>
  );
}
--- Group Sizes
The group size applies to every avatar.
function AvatarGroupSizes(): Node {
  return (
    <View className="flex-col items-start gap-3">
      <AvatarGroup size="small"><Avatar name="Ada" /><Avatar name="Alan" /><Avatar name="Grace" /></AvatarGroup>
      <AvatarGroup size="large"><Avatar name="Ada" /><Avatar name="Alan" /><Avatar name="Grace" /></AvatarGroup>
    </View>
  );
}
=== Alert | alert | Feedback
Inline messages with a variant icon, optional title and close button.
--- Basic Alert
An info alert.
function BasicAlert(): Node {
  return <Alert>This is a simple alert message.</Alert>;
}
--- Alert with Title
AlertTitle and AlertDescription children, as in shadcn/ui. Plain text children are the description.
function TitledAlert(): Node {
  return (
    <Alert>
      <AlertTitle>Heads up!</AlertTitle>
      <AlertDescription>You can add components to your app using the cli.</AlertDescription>
    </Alert>
  );
}
--- Alert Variants
Default, info, success, warning and error.
function AlertVariants(): Node {
  return (
    <View className="flex-col gap-3 w-full">
      <Alert variant="default">This is a default alert.</Alert>
      <Alert variant="info">This is an info alert.</Alert>
      <Alert variant="success">Your changes have been saved.</Alert>
      <Alert variant="warning">Please review your settings.</Alert>
      <Alert variant="error">Something went wrong.</Alert>
    </View>
  );
}
--- Alert Sizes
xsmall to large.
function AlertSizes(): Node {
  return (
    <View className="flex-col gap-3 w-full">
      <Alert size="xsmall">Extra small alert</Alert>
      <Alert size="small">Small alert</Alert>
      <Alert>Medium alert</Alert>
      <Alert size="large">Large alert</Alert>
    </View>
  );
}
--- Closable Alerts
onClose shows a close button.
function ClosableAlert(): Node {
  const open = signal(true);
  return (
    <View className="flex-col items-start gap-3 w-full">
      <Alert variant="info" visible={open.get()} onClose={() => { open.set(false); }} className="w-full">This alert can be closed.</Alert>
      <Button size="small" onPress={() => { open.set(true); }}>Reset</Button>
    </View>
  );
}
--- Banner Mode
Full width, without rounding.
function BannerAlert(): Node {
  return <Alert variant="info" banner={true}>This is a banner alert that spans the full width.</Alert>;
}
--- Custom Icons
Replace the variant icon.
function IconAlert(): Node {
  return <Alert variant="info" icon="bell">You have new notifications.</Alert>;
}
--- Conditional Visibility
visible hides the alert without removing it.
function ConditionalAlert(): Node {
  const show = signal(true);
  return (
    <View className="flex-col gap-3 w-full">
      <Switch checked={show.get()} onChange={(on) => { show.set(on); }}>Show alert</Switch>
      <Alert variant="warning" visible={show.get()}>Shown while the switch is on.</Alert>
    </View>
  );
}
=== Icon | icon | Data Display
Any of the bundled Lucide icons, or inline SVG.
--- Basic Icon
An icon by name.
function BasicIcon(): Node {
  return <Icon name="heart" />;
}
--- Icon with Custom Size
xs, sm, md and lg.
function IconSizes(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Icon name="heart" size="xs" />
      <Icon name="heart" size="sm" />
      <Icon name="heart" />
      <Icon name="heart" size="lg" />
    </View>
  );
}
--- Icon with Custom Color
Text color utilities tint the icon: theme tokens, the Tailwind palette or any [#hex] value.
function ColoredIcons(): Node {
  return (
    <View className="flex-row gap-3">
      <Icon name="heart" className="text-destructive" />
      <Icon name="circle-check" className="text-emerald-500" />
      <Icon name="star" className="text-amber-400" />
      <Icon name="info" className="text-[#1f6feb]" />
    </View>
  );
}
--- Rotated Icons
Rotation in degrees.
function RotatedIcons(): Node {
  return (
    <View className="flex-row gap-3">
      <Icon name="arrow-up" />
      <Icon name="arrow-up" rotate={90} />
      <Icon name="arrow-up" rotate={180} />
      <Icon name="arrow-up" rotate={270} />
    </View>
  );
}
--- SVG Bytes
Inline SVG markup draws as an icon.
function SvgIcon(): Node {
  return <Icon svg="<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><circle cx='12' cy='12' r='9' fill='none' stroke='black' stroke-width='2'/><path d='M8 12l3 3 5-6' fill='none' stroke='black' stroke-width='2'/></svg>" size="lg" />;
}
--- Status Icons
Common status icons.
function StatusIcons(): Node {
  return (
    <View className="flex-row gap-3">
      <Icon name="circle-check" className="text-primary" />
      <Icon name="circle-x" className="text-destructive" />
      <Icon name="triangle-alert" className="text-muted-foreground" />
      <Icon name="info" className="text-foreground" />
    </View>
  );
}
=== Shimmer | shimmer | Feedback
Text with a moving highlight, for work in progress.
--- Basic Usage
A repeating sweep.
function BasicShimmer(): Node {
  return <ShimmerText>Thinking...</ShimmerText>;
}
--- Duration
Milliseconds per sweep.
function ShimmerDuration(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText duration={800}>Fast shimmer</ShimmerText>
      <ShimmerText duration={3000}>Slow shimmer</ShimmerText>
    </View>
  );
}
--- Highlight Color
Any theme color or hex.
function ShimmerColor(): Node {
  return <ShimmerText highlightColor="blue">Generating response</ShimmerText>;
}
--- Spread
Highlight half-width as a fraction of the text.
function ShimmerSpread(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText spread={0.1}>Narrow highlight</ShimmerText>
      <ShimmerText spread={0.5}>Wide highlight</ShimmerText>
    </View>
  );
}
--- Direction and Play-once
Reverse the sweep or play it once.
function ShimmerDirection(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText reverse={true}>Right to left</ShimmerText>
      <ShimmerText once={true}>Plays once</ShimmerText>
    </View>
  );
}
--- Styling
Text utilities style the shimmer.
function StyledShimmer(): Node {
  return <ShimmerText className="text-xl font-semibold">Searching the web</ShimmerText>;
}
=== Empty | empty | Feedback
Empty states with media, a title, a description and actions.
--- Basic Usage
EmptyHeader holds the media, title and description; EmptyContent holds actions. The icon, title and description props are a shorthand.
function BasicEmpty(): Node {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyMedia variant="icon"><Icon name="folder" /></EmptyMedia>
        <EmptyTitle>No projects yet</EmptyTitle>
        <EmptyDescription>Create your first project to get started.</EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <View className="flex-row gap-2">
          <Button variant="primary">Create project</Button>
          <Button outline={true}>Import</Button>
        </View>
      </EmptyContent>
    </Empty>
  );
}
--- Outline
A dashed border.
function OutlineEmpty(): Node {
  return <Empty icon="cloud" title="Cloud storage empty" description="Upload files to see them here." className="border border-dashed rounded-lg" />;
}
--- Background
A muted fill.
function BackgroundEmpty(): Node {
  return <Empty icon="bell" title="No notifications" description="You're all caught up." className="bg-muted rounded-lg" />;
}
--- Avatar
Custom media above the title.
function AvatarEmpty(): Node {
  return (
    <Empty title="User offline" description="This user is currently offline.">
      <EmptyMedia><Avatar name="Ada Lovelace" size="large" /></EmptyMedia>
      <Button size="small">Leave a message</Button>
    </Empty>
  );
}
--- Avatar Group
A group as media.
function GroupEmpty(): Node {
  return (
    <Empty title="No team members" description="Invite your team to collaborate.">
      <EmptyMedia>
        <AvatarGroup><Avatar name="Ada" /><Avatar name="Alan" /><Avatar name="Grace" /></AvatarGroup>
      </EmptyMedia>
      <Button variant="primary" size="small">Invite members</Button>
    </Empty>
  );
}
--- Inputs and Custom Content
Any content as the body.
function InputEmpty(): Node {
  return (
    <Empty icon="search" title="404 - Not Found" description="The page you're looking for doesn't exist.">
      <Input placeholder="Search pages..." className="w-64" />
    </Empty>
  );
}
=== GroupBox | group-box | Layout
A titled surface grouping related content, with an optional footer.
--- Basic GroupBox
Content on a surface.
function BasicGroupBox(): Node {
  return (
    <GroupBox>
      <Text>One</Text>
      <Text>Two</Text>
      <Text>Three</Text>
    </GroupBox>
  );
}
--- GroupBox Variants
Normal, fill and outline.
function GroupBoxVariants(): Node {
  return (
    <View className="flex-col gap-4 w-full">
      <GroupBox title="Normal"><Text>Default surface</Text></GroupBox>
      <GroupBox title="Fill" variant="fill"><Text>Filled surface</Text></GroupBox>
      <GroupBox title="Outline" variant="outline"><Text>Outlined surface</Text></GroupBox>
    </View>
  );
}
--- Footer Outside the Surface
A GroupBoxFooter child sits below the surface.
function FooterGroupBox(): Node {
  return (
    <GroupBox title="Notifications" variant="fill">
      <Switch defaultChecked={true}>Email</Switch>
      <Switch>Push</Switch>
      <GroupBoxFooter><Button variant="primary" size="small">Save</Button></GroupBoxFooter>
    </GroupBox>
  );
}
--- Custom Title Styling
Utility classes for the title.
function TitleStyledGroupBox(): Node {
  return (
    <GroupBox title="Account" titleClassName="text-xl font-semibold text-primary">
      <Text>Signed in as ada@example.com</Text>
    </GroupBox>
  );
}
--- Custom Content Styling
Utility classes for the content surface.
function ContentStyledGroupBox(): Node {
  return (
    <GroupBox title="Spacing" variant="outline" contentClassName="gap-4 p-6 rounded-xl">
      <Text>Roomier content</Text>
      <Text>with larger gaps</Text>
    </GroupBox>
  );
}
--- Form Section
Fields grouped under a heading.
function FormGroupBox(): Node {
  return (
    <GroupBox title="Profile" variant="fill">
      <Input placeholder="Name" />
      <Input placeholder="Email" />
      <Checkbox>Subscribe to updates</Checkbox>
    </GroupBox>
  );
}
=== Clipboard | clipboard | Actions
A copy button for a value, with copied feedback.
--- Basic Clipboard
Copies the value.
function BasicClipboard(): Node {
  return (
    <View className="flex-row items-center gap-2">
      <Text>npm install tsrust</Text>
      <Clipboard value="npm install tsrust" />
    </View>
  );
}
--- Using Dynamic Values
The value follows state.
function DynamicClipboard(): Node {
  const count = signal(1);
  return (
    <View className="flex-row items-center gap-2">
      <Button size="small" onPress={() => { count.set(count.get() + 1); }}>Next</Button>
      <Text>Item {count}</Text>
      <Clipboard value={`item-${count.get()}`} />
    </View>
  );
}
--- Copied Callback and Tooltip
onCopied receives the copied text.
function CopiedClipboard(): Node {
  const copied = signal('');
  return (
    <View className="flex-row items-center gap-2">
      <Clipboard value="sk-example-key" tooltip="Copy key" tooltipDelay={200} accessibilityLabel="Copy API key" onCopied={(text) => { copied.set(text); }} />
      <Text>{copied.get() === '' ? 'Not copied' : `Copied ${copied.get()}`}</Text>
    </View>
  );
}
--- Sizes
small to large.
function ClipboardSizes(): Node {
  return (
    <View className="flex-row items-center gap-3">
      <Clipboard value="small" size="small" />
      <Clipboard value="medium" />
      <Clipboard value="large" size="large" />
    </View>
  );
}
=== Checkbox | checkbox | Forms
A checked or unchecked choice with an optional label.
--- Basic Checkbox
Controlled with checked and onChange. Without checked a checkbox keeps its own state; defaultChecked sets where it starts.
function BasicCheckbox(): Node {
  const checked = signal(false);
  return <Checkbox checked={checked.get()} onChange={(on) => { checked.set(on); }}>Accept terms and conditions</Checkbox>;
}
--- Different Sizes
xsmall to large.
function CheckboxSizes(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox size="xsmall">Extra Small</Checkbox>
      <Checkbox size="small">Small</Checkbox>
      <Checkbox>Medium</Checkbox>
      <Checkbox size="large">Large</Checkbox>
    </View>
  );
}
--- Disabled State
Disabled checkboxes ignore input.
function DisabledCheckbox(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox disabled={true}>Disabled checkbox</Checkbox>
      <Checkbox checked={true} disabled={true}>Disabled checked</Checkbox>
    </View>
  );
}
--- Without Label
An accessibility label names a bare checkbox.
function BareCheckbox(): Node {
  const checked = signal(true);
  return <Checkbox accessibilityLabel="Remember me" checked={checked.get()} onChange={(on) => { checked.set(on); }} />;
}
--- Tooltip and Role
A tooltip, or the switch role for on/off settings.
function TooltipCheckbox(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox tooltip="Sync across devices" tooltipDelay={300}>Sync</Checkbox>
      <Checkbox role="switch">Dark mode</Checkbox>
    </View>
  );
}
--- Custom Tab Order
tabIndex orders focus; tabStop skips a control.
function TabOrderCheckbox(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox tabIndex={2}>Second</Checkbox>
      <Checkbox tabIndex={1}>First</Checkbox>
      <Checkbox tabStop={false}>Skipped</Checkbox>
    </View>
  );
}
=== Switch | switch | Forms
An on/off toggle for settings that apply immediately.
--- Basic Switch
Controlled with checked and onChange.
function BasicSwitch(): Node {
  const on = signal(false);
  return <Switch checked={on.get()} onChange={(value) => { on.set(value); }} />;
}
--- With Label
A label beside the switch.
function LabeledSwitch(): Node {
  const on = signal(true);
  return <Switch checked={on.get()} onChange={(value) => { on.set(value); }}>Enable notifications</Switch>;
}
--- Without Label
An accessibility label names a bare switch.
function BareSwitch(): Node {
  return <Switch accessibilityLabel="Airplane mode" />;
}
--- Different Sizes
small and medium.
function SwitchSizes(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch size="small" defaultChecked={true} />
      <Switch defaultChecked={true} />
    </View>
  );
}
--- Disabled State
Disabled switches keep their state.
function DisabledSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch disabled={true}>Disabled off</Switch>
      <Switch checked={true} disabled={true}>Disabled on</Switch>
    </View>
  );
}
--- Custom Color
The track color when on; disabled draws it at half opacity.
function ColoredSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch defaultChecked={true} color="success" />
      <Switch defaultChecked={true} color="danger" />
      <Switch checked={true} color="success" disabled={true} />
    </View>
  );
}
--- With Tooltip
A tooltip on hover.
function TooltipSwitch(): Node {
  return <Switch tooltip="Save changes automatically">Auto-save</Switch>;
}
--- Keyboard Focus
tabIndex orders focus; tabStop skips it.
function FocusSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch tabIndex={1}>Focusable</Switch>
      <Switch tabStop={false}>Skipped</Switch>
    </View>
  );
}
=== Radio | radio | Forms
One choice from a set, alone or in a RadioGroup.
--- Basic Radio Button
A standalone radio.
function BasicRadio(): Node {
  const checked = signal(false);
  return <Radio checked={checked.get()} onChange={(on) => { checked.set(on); }}>Option 1</Radio>;
}
--- Radio Group
The group tracks the selected index.
function BasicRadioGroup(): Node {
  const selected = signal(0);
  return (
    <RadioGroup selected={selected.get()} onChange={(index) => { selected.set(index); }}>
      <Radio>Option 1</Radio>
      <Radio>Option 2</Radio>
      <Radio>Option 3</Radio>
    </RadioGroup>
  );
}
--- Different Sizes
xsmall to large.
function RadioSizes(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio size="xsmall">Extra small</Radio>
      <Radio size="small">Small</Radio>
      <Radio>Medium</Radio>
      <Radio size="large">Large</Radio>
    </View>
  );
}
--- Disabled State
Disabled radios ignore input.
function DisabledRadio(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio disabled={true}>Disabled</Radio>
      <Radio checked={true} disabled={true}>Disabled checked</Radio>
    </View>
  );
}
--- Tooltip and Tab Order
Tooltips, accessibility names and focus order.
function TooltipRadio(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio tooltip="Arrives tomorrow" tabIndex={1}>Express</Radio>
      <Radio accessibilityLabel="Standard shipping" tabStop={false}>Standard</Radio>
    </View>
  );
}
--- Vertical Layout
Options stacked vertically.
function VerticalRadioGroup(): Node {
  const selected = signal(1);
  return (
    <RadioGroup vertical={true} selected={selected.get()} onChange={(index) => { selected.set(index); }}>
      <Radio>Small</Radio>
      <Radio>Medium</Radio>
      <Radio>Large</Radio>
    </RadioGroup>
  );
}
--- Disabled Radio Group
A disabled group disables every option.
function DisabledRadioGroup(): Node {
  return (
    <RadioGroup vertical={true} selected={0} disabled={true}>
      <Radio>Option 1</Radio>
      <Radio>Option 2</Radio>
    </RadioGroup>
  );
}
=== Rating | rating | Forms
A star rating input.
--- Basic Rating
Controlled with value and onChange.
function BasicRating(): Node {
  const value = signal(3);
  return <Rating value={value.get()} onChange={(stars) => { value.set(stars); }} />;
}
--- Different Sizes
xsmall to large.
function RatingSizes(): Node {
  return (
    <View className="flex-col gap-2">
      <Rating size="xsmall" defaultValue={3} />
      <Rating size="small" defaultValue={3} />
      <Rating defaultValue={3} />
      <Rating size="large" defaultValue={3} />
    </View>
  );
}
--- Custom Color
A theme color, palette color or hex.
function ColoredRating(): Node {
  return <Rating defaultValue={4} color="danger" />;
}
--- Disabled State
A read-only display.
function DisabledRating(): Node {
  return <Rating value={2} disabled={true} />;
}
--- Custom Maximum
Any number of stars.
function MaxRating(): Node {
  const value = signal(7);
  return <Rating value={value.get()} max={10} onChange={(stars) => { value.set(stars); }} />;
}
=== Input | input | Forms
A single-line text field with placeholders, masking, input masks, validation, prefix and suffix icons, sizes and states. Use Textarea for multiple lines and InputGroup for addons.
--- Basic Input
Uncontrolled: the field keeps its own text. onChange reports each edit.
function BasicInput(): Node {
  const text = signal('');
  return (
    <View className="flex-col gap-2 w-80">
      <Input onChange={(event) => { text.set(event.target.value); }} />
      <Text variant="muted">Value: {text}</Text>
    </View>
  );
}
--- With Placeholder
Hint text while the field is empty.
function PlaceholderInput(): Node {
  return <Input placeholder="Enter your name..." className="w-80" />;
}
--- With Default Value
The starting text of an uncontrolled field.
function DefaultValueInput(): Node {
  return <Input defaultValue="John Doe" className="w-80" />;
}
--- Cleanable Input
A clear button while the field has a value.
function CleanableInput(): Node {
  return <Input cleanable={true} defaultValue="Clear me" className="w-80" />;
}
--- With Prefix and Suffix
Icons before and after the text. For buttons and text addons, use InputGroup.
function PrefixSuffixInput(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Input prefix="search" placeholder="Search..." />
      <Input suffix="info" placeholder="With a suffix" />
      <Input prefix="search" suffix="info" placeholder="Both" />
    </View>
  );
}
--- Password Input (Masked)
type="password" masks the value and adds a reveal toggle; copy and cut are blocked while masked.
function PasswordInput(): Node {
  return <Input type="password" defaultValue="password123" className="w-80" />;
}
--- Input Sizes
xsmall to large.
function InputSizes(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Input size="large" placeholder="Large" />
      <Input placeholder="Medium" />
      <Input size="small" placeholder="Small" />
      <Input size="xsmall" placeholder="Extra small" />
    </View>
  );
}
--- Disabled and Read-only
A read-only field keeps its look and can still be focused, selected and copied.
function DisabledInput(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Input disabled={true} defaultValue="Disabled" />
      <Input readOnly={true} defaultValue="Read-only: select and copy me" />
    </View>
  );
}
--- Clean on ESC
Escape clears the field.
function EscapeInput(): Node {
  return <Input cleanOnEscape={true} placeholder="Type, then press Escape" className="w-80" />;
}
--- Input Validation
A pattern every value must match; edits that would break it are rejected.
function ValidatedInput(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Input pattern="[0-9]*[.]?[0-9]*" placeholder="Numbers only" />
      <Input pattern="[a-zA-Z0-9]*" placeholder="Letters and digits only" />
    </View>
  );
}
--- Input Masking
mask formats as you type: 9 an optional digit, # a digit, A a letter. thousandsSeparator and fractionDigits format numbers.
function MaskedInput(): Node {
  const amount = signal('');
  return (
    <View className="flex-col gap-2 w-80">
      <Input mask="(999)-999-9999" placeholder="Phone" />
      <Input mask="AAA-###-AAA" placeholder="Code" />
      <Input thousandsSeparator="," fractionDigits={2} placeholder="Amount" onChange={(event) => { amount.set(event.target.value); }} />
      <Text variant="muted">Value: {amount}</Text>
    </View>
  );
}
--- Handle Input Events
onFocus, onBlur and onSubmit (Enter) beside onChange.
function InputEvents(): Node {
  const log = signal('Nothing yet');
  return (
    <View className="flex-col gap-2 w-80">
      <Input placeholder="Focus, type and press Enter" onFocus={() => { log.set('Focused'); }} onBlur={() => { log.set('Blurred'); }} onSubmit={(secondary) => { log.set(secondary ? 'Submitted with the modifier' : 'Submitted'); }} />
      <Text variant="muted">{log}</Text>
    </View>
  );
}
--- Custom Appearance
appearance=false draws bare text for a custom container.
function BareInput(): Node {
  return (
    <View className="w-80 border-b-2 border-border bg-secondary px-6 py-3">
      <Input appearance={false} placeholder="Borderless" />
    </View>
  );
}
--- Context Menu
contextMenu=false turns off the right-click edit menu.
function NoMenuInput(): Node {
  return <Input contextMenu={false} defaultValue="Right-click does nothing" className="w-80" />;
}
--- Programmatic Control
controlInput runs actions on the field with that id; readInput reads its text and selection.
function ControlledInput(): Node {
  const selection = signal('Select some text, then read it');
  return (
    <View className="flex-col gap-2 w-96">
      <Input id="demo-field" defaultValue="The quick brown fox" />
      <View className="flex-row flex-wrap gap-2">
        <Button size="small" onPress={() => { controlInput('demo-field', 'focus'); }}>Focus</Button>
        <Button size="small" onPress={() => { controlInput('demo-field', 'selectAll'); }}>Select all</Button>
        <Button size="small" onPress={() => { controlInput('demo-field', 'insert', { value: ' jumps' }); }}>Insert</Button>
        <Button size="small" onPress={() => { controlInput('demo-field', 'setSelection', { start: 4, end: 9 }); }}>Select "quick"</Button>
        <Button size="small" onPress={() => { controlInput('demo-field', 'clear'); }}>Clear</Button>
        <Button size="small" onPress={() => { readInput('demo-field', (field) => { selection.set(`Selected: "${field.selectedText ?? ''}" (${field.selectionStart ?? 0}-${field.selectionEnd ?? 0})`); }); }}>Read selection</Button>
      </View>
      <Text variant="muted">{selection}</Text>
    </View>
  );
}
--- Atomic Inline Tokens
Tokens select and delete as a whole; onTokenClick receives the token id.
function TokenInput(): Node {
  const clicked = signal('none');
  return (
    <View className="flex-col gap-2 w-96">
      <Input id="token-field" placeholder="Message" onTokenClick={(id) => { clicked.set(id); }} />
      <View className="flex-row gap-2">
        <Button size="small" onPress={() => { controlInput('token-field', 'insertToken', { value: '@alice', label: 'Alice' }); }}>Mention Alice</Button>
        <Button size="small" onPress={() => { controlInput('token-field', 'insertToken', { value: '#general', label: 'general' }); }}>Add #general</Button>
      </View>
      <Text variant="muted">Clicked token: {clicked}</Text>
    </View>
  );
}
=== Textarea | textarea | Forms
A multi-line text field with fixed or auto-growing height, find and replace, and the Input's masking, events and control.
--- Basic Usage
Four rows by default.
function BasicTextarea(): Node {
  return <Textarea placeholder="Write a note..." className="w-96" />;
}
--- Rows
A fixed row count.
function RowsTextarea(): Node {
  return <Textarea rows={8} placeholder="Eight rows" className="w-96" />;
}
--- Auto-grow
Grows with its content between minRows and maxRows, then scrolls.
function GrowingTextarea(): Node {
  return <Textarea minRows={2} maxRows={6} placeholder="Type several lines" className="w-96" />;
}
--- Scroll Bounce
Elastic overscroll at the edges.
function BounceTextarea(): Node {
  return <Textarea scrollBounce={true} rows={3} defaultValue={'One\nTwo\nThree\nFour\nFive\nSix\nSeven'} className="w-96" />;
}
--- Value and Events
Controlled text with a character count; Enter inserts newlines.
function ControlledTextarea(): Node {
  const text = signal('');
  return (
    <View className="flex-col gap-2 w-96">
      <Textarea value={text} onChange={(event) => { text.set(event.target.value); }} placeholder="Type here" />
      <Text variant="muted">{text.get().length} characters</Text>
    </View>
  );
}
--- Sizes
Size changes the text and padding together.
function TextareaSizes(): Node {
  return (
    <View className="flex-col gap-2 w-96">
      <Textarea size="small" rows={2} placeholder="Small" />
      <Textarea rows={2} placeholder="Medium" />
      <Textarea size="large" rows={2} placeholder="Large" />
    </View>
  );
}
--- Appearance
Disabled, read-only and bare.
function TextareaStates(): Node {
  return (
    <View className="flex-col gap-2 w-96">
      <Textarea disabled={true} rows={2} defaultValue="Disabled" />
      <Textarea readOnly={true} rows={2} defaultValue="Read-only text keeps the normal look." />
      <Textarea appearance={false} rows={2} placeholder="No border or background" />
    </View>
  );
}
--- Wrapping and Whitespace
softWrap=false scrolls long lines; showWhitespace marks spaces and tabs.
function WrapTextarea(): Node {
  return (
    <View className="flex-col gap-2 w-96">
      <Textarea softWrap={false} rows={2} defaultValue="A long line that keeps going past the edge of the field instead of wrapping onto the next row." />
      <Textarea showWhitespace={true} tabSize={4} rows={2} defaultValue={'\tIndented with a tab\n    and with spaces'} />
    </View>
  );
}
--- Find and Replace
searchable turns on cmd/ctrl-F; controlInput runs search actions.
function SearchTextarea(): Node {
  return (
    <View className="flex-col gap-2 w-96">
      <Textarea id="search-notes" searchable={true} rows={4} defaultValue={'apples and oranges\nmore apples\nno fruit here\napples again'} />
      <View className="flex-row flex-wrap gap-2">
        <Button size="small" onPress={() => { controlInput('search-notes', 'search', { value: 'apples' }); }}>Find apples</Button>
        <Button size="small" onPress={() => { controlInput('search-notes', 'nextMatch'); }}>Next</Button>
        <Button size="small" onPress={() => { controlInput('search-notes', 'replaceAllMatches', { value: 'pears' }); }}>Replace all</Button>
        <Button size="small" onPress={() => { controlInput('search-notes', 'closeSearch'); }}>Close</Button>
      </View>
    </View>
  );
}
=== Number Input | number-input | Forms
A numeric field with increment and decrement buttons, bounds and steps.
--- Basic Number Input
Controlled with value and onChange.
function BasicNumberInput(): Node {
  const value = signal<number>(10);
  return (
    <View className="flex-col gap-2 w-60">
      <NumberInput value={value.get()} onChange={(next) => { value.set(next); }} />
      <Text variant="muted">Value: {value}</Text>
    </View>
  );
}
--- With Min/Max/Step
Stepping clamps to the bounds; a typed value out of range is clamped on blur.
function BoundedNumberInput(): Node {
  return <NumberInput defaultValue={50} min={0} max={100} step={5} className="w-60" />;
}
--- Different Sizes
xsmall to large.
function NumberInputSizes(): Node {
  return (
    <View className="flex-col gap-2 w-60">
      <NumberInput defaultValue={1} size="large" />
      <NumberInput defaultValue={1} />
      <NumberInput defaultValue={1} size="small" />
      <NumberInput defaultValue={1} size="xsmall" />
    </View>
  );
}
--- With Prefix and Suffix
Icons beside the number.
function AffixNumberInput(): Node {
  return <NumberInput defaultValue={25} prefix="dollar-sign" suffix="percent" className="w-60" />;
}
--- Disabled State
Ignores typing and stepping.
function DisabledNumberInput(): Node {
  return <NumberInput defaultValue={3} disabled={true} className="w-60" />;
}
--- Accessible Name
Names a field that has no visible label.
function NamedNumberInput(): Node {
  return <NumberInput defaultValue={2} accessibilityLabel="Quantity" placeholder="Quantity" className="w-60" />;
}
--- Without Default Styling
appearance=false draws the bare field.
function BareNumberInput(): Node {
  return (
    <View className="w-60 border-b-2 border-border px-3 py-2">
      <NumberInput defaultValue={7} appearance={false} />
    </View>
  );
}
=== Input Group | input-group | Forms
An Input or Textarea framed with addons: text, icons and buttons before, after, above or below.
--- Input with a Clear Button
An icon addon, a live character count and a clear button.
function ClearableGroup(): Node {
  const query = signal('');
  return (
    <InputGroup className="w-96">
      <InputGroupAddon><Icon name="search" /></InputGroupAddon>
      <Input id="group-search" placeholder="Search..." value={query} onChange={(event) => { query.set(event.target.value); }} />
      <InputGroupAddon align="inline-end">
        <InputGroupText>{query.get().length} characters</InputGroupText>
        <InputGroupButton disabled={query.get().length === 0} onClick={() => { query.set(''); controlInput('group-search', 'focus'); }}>Clear</InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  );
}
--- Text Addons
A protocol before and a domain after the text.
function TextAddonGroup(): Node {
  return (
    <InputGroup className="w-96">
      <InputGroupAddon><InputGroupText>https://</InputGroupText></InputGroupAddon>
      <Input placeholder="example" />
      <InputGroupAddon align="inline-end"><InputGroupText>.com</InputGroupText></InputGroupAddon>
    </InputGroup>
  );
}
--- Buttons, Icons and Menus
Icon buttons with tooltips, variants, and a button that opens a menu.
function ButtonAddonGroup(): Node {
  const engine = signal('Web');
  return (
    <InputGroup className="w-96">
      <Input placeholder="Search the web" />
      <InputGroupAddon align="inline-end">
        <InputGroupButton icon="x" accessibilityLabel="Clear search" tooltip="Clear search" />
        <InputGroupButton caret={true}>
          {engine}
          <MenuItem onSelect={() => { engine.set('Web'); }}>Web</MenuItem>
          <MenuItem onSelect={() => { engine.set('Images'); }}>Images</MenuItem>
        </InputGroupButton>
        <InputGroupButton variant="primary">Go</InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  );
}
--- Textarea with a Counter and Submit
Block addons sit above and below a growing textarea.
function ComposerGroup(): Node {
  const draft = signal('');
  const sent = signal('');
  return (
    <View className="flex-col gap-2 w-96">
      <InputGroup>
        <Textarea id="composer" minRows={2} maxRows={6} placeholder="Write a message" value={draft} onChange={(event) => { draft.set(event.target.value); }} />
        <InputGroupAddon align="block-end">
          <InputGroupText>{draft.get().length}/280</InputGroupText>
          <View className="grow" />
          <InputGroupButton variant="primary" disabled={draft.get().length === 0 || draft.get().length > 280} onClick={() => { sent.set(draft.get()); draft.set(''); controlInput('composer', 'focus'); }}>Send</InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
      <Text variant="muted">Sent: {sent}</Text>
    </View>
  );
}
--- States
Disabled, read-only and invalid groups.
function GroupStates(): Node {
  return (
    <View className="flex-col gap-2 w-96">
      <InputGroup disabled={true}><Input defaultValue="Disabled" /><InputGroupAddon align="inline-end"><InputGroupButton>Go</InputGroupButton></InputGroupAddon></InputGroup>
      <InputGroup readOnly={true}><Input readOnly={true} defaultValue="Read-only" /></InputGroup>
      <InputGroup invalid={true}><InputGroupAddon><Icon name="circle-alert" /></InputGroupAddon><Input defaultValue="Invalid value" /></InputGroup>
    </View>
  );
}
=== Select | select | Forms
Choose one option from a list, with search, groups, sizes and states. For custom triggers or several choices, see Combobox.
--- Basic Select
SelectItem children; the select keeps its own choice, starting from defaultValue.
function BasicSelect(): Node {
  return (
    <Select defaultValue="apple" className="w-60">
      <SelectItem value="apple">Apple</SelectItem>
      <SelectItem value="orange">Orange</SelectItem>
      <SelectItem value="banana">Banana</SelectItem>
    </Select>
  );
}
--- Placeholder
Shown until something is chosen.
function PlaceholderSelect(): Node {
  return <Select placeholder="Select a language..." items={['Rust', 'Go', 'JavaScript']} className="w-60" />;
}
--- Accessibility
A name that stays the same as the selection changes.
function NamedSelect(): Node {
  return <Select accessibilityLabel="Programming language" placeholder="Choose a language" items={['Rust', 'TypeScript', 'Python']} className="w-60" />;
}
--- Searchable
A search field filters the options.
function SearchableSelect(): Node {
  return <Select searchable={true} icon="search" searchPlaceholder="Search fruit" placeholder="Pick a fruit" items={['Apple', 'Orange', 'Banana', 'Grape', 'Pineapple']} className="w-60" />;
}
--- Values and Labels
value is what onChange reports; the children are what people see.
function ValueSelect(): Node {
  const country = signal('');
  return (
    <View className="flex-col gap-2 w-60">
      <Select placeholder="Country" value={country.get()} onChange={(value) => { country.set(value); }}>
        <SelectItem value="US">United States</SelectItem>
        <SelectItem value="CA">Canada</SelectItem>
        <SelectItem value="MX">Mexico</SelectItem>
        <SelectItem value="GL" disabled={true}>Greenland</SelectItem>
      </Select>
      <Text variant="muted">Value: {country}</Text>
    </View>
  );
}
--- Group Items
SelectGroup with a SelectLabel title.
function GroupedSelect(): Node {
  return (
    <Select placeholder="Select a country" searchable={true} className="w-60">
      <SelectGroup>
        <SelectLabel>A</SelectLabel>
        <SelectItem value="AU">Australia</SelectItem>
        <SelectItem value="AT">Austria</SelectItem>
      </SelectGroup>
      <SelectGroup>
        <SelectLabel>B</SelectLabel>
        <SelectItem value="BR">Brazil</SelectItem>
        <SelectItem value="BE">Belgium</SelectItem>
      </SelectGroup>
    </Select>
  );
}
--- Sizes
xsmall to large.
function SelectSizes(): Node {
  return (
    <View className="flex-col gap-2 w-60">
      <Select size="large" placeholder="Large" items={['One', 'Two']} />
      <Select placeholder="Medium" items={['One', 'Two']} />
      <Select size="small" placeholder="Small" items={['One', 'Two']} />
    </View>
  );
}
--- Disabled State
Ignores clicks and keys.
function DisabledSelect(): Node {
  return <Select disabled={true} defaultValue="Rust" items={['Rust', 'Go']} className="w-60" />;
}
--- Cleanable
A clear button while something is selected.
function CleanableSelect(): Node {
  return <Select cleanable={true} defaultValue="Go" items={['Rust', 'Go', 'Zig']} className="w-60" />;
}
--- Custom Appearance
A prefix before the label, a wider menu with a height limit, or a bare trigger.
function StyledSelect(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Select titlePrefix="Country: " menuWidth={320} menuMaxHeight={160} defaultValue="Canada" items={['Canada', 'Chile', 'China', 'Colombia', 'Croatia', 'Cuba', 'Cyprus']} />
      <View className="border-b-2 border-border bg-secondary px-4 py-2">
        <Select appearance={false} placeholder="Borderless" items={['One', 'Two']} />
      </View>
    </View>
  );
}
--- Empty State
SelectEmpty replaces the list when there is nothing to show.
function EmptySelect(): Node {
  return (
    <Select placeholder="No options" className="w-60">
      <SelectEmpty><Text variant="muted">No options available</Text></SelectEmpty>
    </Select>
  );
}
--- Controlled Selection
The selected value follows state.
function DynamicSelect(): Node {
  const size = signal('M');
  return (
    <View className="flex-col gap-2 w-80">
      <Select value={size.get()} onChange={(value) => { size.set(value); }} items={['S', 'M', 'L', 'XL']} />
      <View className="flex-row gap-2">
        <Button size="small" onPress={() => { size.set('L'); }}>Choose L</Button>
        <Button size="small" onPress={() => { size.set('S'); }}>Choose S</Button>
      </View>
      <Text variant="muted">Size: {size}</Text>
    </View>
  );
}
=== Combobox | combobox | Forms
A searchable picker for one or several options, with groups, a custom trigger and a footer. Use Select for a plain single choice.
--- Basic Single-Select
Search filters the options; onChange receives the selected values.
function BasicCombobox(): Node {
  const framework = signal('');
  return (
    <View className="flex-col gap-2 w-72">
      <Combobox placeholder="Select framework..." searchPlaceholder="Search..." onChange={(values) => { framework.set(values.length > 0 ? values[0] : ''); }}>
        <SelectItem value="next">Next.js</SelectItem>
        <SelectItem value="svelte">SvelteKit</SelectItem>
        <SelectItem value="nuxt">Nuxt.js</SelectItem>
        <SelectItem value="remix">Remix</SelectItem>
        <SelectItem value="astro">Astro</SelectItem>
      </Combobox>
      <Text variant="muted">Selected: {framework}</Text>
    </View>
  );
}
--- Multi-Select
multiple toggles each option; the list stays open.
function MultiCombobox(): Node {
  const count = signal(0);
  return (
    <View className="flex-col gap-2 w-72">
      <Combobox multiple={true} placeholder="Select frameworks" items={['Next.js', 'SvelteKit', 'Nuxt.js', 'Remix', 'Astro']} onChange={(values) => { count.set(values.length); }} />
      <Text variant="muted">{count} selected</Text>
    </View>
  );
}
--- Pre-selected Items
defaultValue starts the selection.
function PreselectedCombobox(): Node {
  return <Combobox multiple={true} defaultValue={['Rust', 'Go']} items={['Rust', 'Go', 'Zig', 'C']} className="w-72" />;
}
--- Grouped Items
SelectGroup and SelectLabel, as in Select.
function GroupedCombobox(): Node {
  return (
    <Combobox placeholder="Select a timezone" className="w-72">
      <SelectGroup>
        <SelectLabel>Americas</SelectLabel>
        <SelectItem value="est">Eastern Time</SelectItem>
        <SelectItem value="pst">Pacific Time</SelectItem>
      </SelectGroup>
      <SelectGroup>
        <SelectLabel>Europe</SelectLabel>
        <SelectItem value="gmt">Greenwich Mean Time</SelectItem>
        <SelectItem value="cet">Central European Time</SelectItem>
      </SelectGroup>
    </Combobox>
  );
}
--- Disabled Items
Disabled options show but cannot be chosen.
function DisabledItemsCombobox(): Node {
  return (
    <Combobox placeholder="Select a plan" className="w-72">
      <SelectItem value="free">Free</SelectItem>
      <SelectItem value="pro">Pro</SelectItem>
      <SelectItem value="enterprise" disabled={true}>Enterprise (contact sales)</SelectItem>
    </Combobox>
  );
}
--- Custom Check Icon
Any icon marks selected options.
function CheckIconCombobox(): Node {
  return <Combobox multiple={true} checkIcon="circle-check" items={['Apple', 'Banana', 'Cherry']} placeholder="Pick fruit" className="w-72" />;
}
--- Footer Action
ComboboxFooter sits under the list.
function FooterCombobox(): Node {
  const teams = signal(2);
  return (
    <Combobox placeholder="Select team" items={['Design', 'Engineering', 'Marketing']} className="w-72">
      <ComboboxFooter><Button size="small" variant="ghost" icon="plus" onPress={() => { teams.set(teams.get() + 1); }}>New team ({teams})</Button></ComboboxFooter>
    </Combobox>
  );
}
--- Custom Trigger
ComboboxTrigger replaces the selected labels.
function TriggerCombobox(): Node {
  return (
    <Combobox items={['Ada Lovelace', 'Grace Hopper', 'Alan Turing']} className="w-72">
      <ComboboxTrigger><Icon name="user" /><Text>Assign people</Text></ComboboxTrigger>
    </Combobox>
  );
}
--- Sizes
small, medium and large.
function ComboboxSizes(): Node {
  return (
    <View className="flex-col gap-2 w-72">
      <Combobox size="large" placeholder="Large" items={['One', 'Two']} />
      <Combobox placeholder="Medium" items={['One', 'Two']} />
      <Combobox size="small" placeholder="Small" items={['One', 'Two']} />
    </View>
  );
}
--- Cleanable and Disabled
A clear button, or no interaction at all.
function CleanableCombobox(): Node {
  return (
    <View className="flex-col gap-2 w-72">
      <Combobox cleanable={true} defaultValue={['Go']} items={['Rust', 'Go']} />
      <Combobox disabled={true} defaultValue={['Rust']} items={['Rust', 'Go']} />
    </View>
  );
}
--- Controlled Selection
value holds the selected values one per line; buttons change it from outside.
function ControlledCombobox(): Node {
  const tags = signal('bug');
  return (
    <View className="flex-col gap-2 w-72">
      <Combobox multiple={true} value={tags.get()} onChange={(values) => { tags.set(values.join('\n')); }} items={['bug', 'feature', 'docs', 'question']} />
      <View className="flex-row gap-2">
        <Button size="small" onPress={() => { tags.set('feature\ndocs'); }}>Set two</Button>
        <Button size="small" onPress={() => { tags.set(''); }}>Clear</Button>
      </View>
      <Text variant="muted">Tags: {tags.get().split('\n').join(', ')}</Text>
    </View>
  );
}
=== Tabs | tabs | Navigation
Tab triggers that switch between content panels, in several styles, sizes and with controls before or after the tabs.
--- Basic Tabs
Tab and TabsContent pair by value; the tabs keep their own selection.
function BasicTabs(): Node {
  return (
    <Tabs defaultValue="account" className="w-96">
      <Tab value="account">Account</Tab>
      <Tab value="profile">Profile</Tab>
      <Tab value="settings">Settings</Tab>
      <TabsContent value="account"><Text>Make changes to your account here.</Text></TabsContent>
      <TabsContent value="profile"><Text>Update your public profile.</Text></TabsContent>
      <TabsContent value="settings"><Text>Choose your preferences.</Text></TabsContent>
    </Tabs>
  );
}
--- Tab Variants
tab (default), underline, pill, outline and segmented.
function TabVariants(): Node {
  return (
    <View className="flex-col gap-4">
      <Tabs><Tab>Account</Tab><Tab>Profile</Tab><Tab>Documents</Tab></Tabs>
      <Tabs variant="underline"><Tab>Account</Tab><Tab>Profile</Tab><Tab>Documents</Tab></Tabs>
      <Tabs variant="pill"><Tab>Account</Tab><Tab>Profile</Tab><Tab>Documents</Tab></Tabs>
      <Tabs variant="outline"><Tab>Account</Tab><Tab>Profile</Tab><Tab>Documents</Tab></Tabs>
      <Tabs variant="segmented"><Tab icon="bot" accessibilityLabel="Bot" /><Tab icon="calendar" accessibilityLabel="Calendar" /><Tab icon="map" accessibilityLabel="Map" /><Tab>Settings</Tab><Tab>About</Tab></Tabs>
    </View>
  );
}
--- Folder Tabs
Browser-style tabs; the selected tab joins the content below. TabSuffix adds a close button.
function FolderTabs(): Node {
  return (
    <Tabs variant="folder" className="w-96">
      <Tab>Account<TabSuffix><Button size="xsmall" variant="ghost" icon="x" /></TabSuffix></Tab>
      <Tab>Profile</Tab>
      <Tab>Documents</Tab>
    </Tabs>
  );
}
--- Tab Sizes
xsmall to large.
function TabSizes(): Node {
  return (
    <View className="flex-col gap-3">
      <Tabs size="xsmall"><Tab>Extra small</Tab><Tab>Tab</Tab></Tabs>
      <Tabs size="small"><Tab>Small</Tab><Tab>Tab</Tab></Tabs>
      <Tabs><Tab>Medium</Tab><Tab>Tab</Tab></Tabs>
      <Tabs size="large"><Tab>Large</Tab><Tab>Tab</Tab></Tabs>
    </View>
  );
}
--- Tabs with Icons
Icon-only tabs need an accessibility label.
function IconTabs(): Node {
  return (
    <Tabs>
      <Tab icon="user" accessibilityLabel="User" />
      <Tab icon="settings" accessibilityLabel="Settings" />
      <Tab icon="mail" accessibilityLabel="Mail" />
    </Tabs>
  );
}
--- Tabs with Prefix and Suffix
Controls before and after the tabs.
function AffixTabs(): Node {
  return (
    <Tabs className="w-full">
      <TabsPrefix><Button size="xsmall" variant="ghost" icon="arrow-left" /><Button size="xsmall" variant="ghost" icon="arrow-right" /></TabsPrefix>
      <Tab>Account</Tab>
      <Tab>Profile</Tab>
      <Tab>Settings</Tab>
      <TabsSuffix><Button size="xsmall" variant="ghost" icon="inbox" /><Button size="xsmall" variant="ghost" icon="ellipsis" /></TabsSuffix>
    </Tabs>
  );
}
--- Disabled Tabs
Disabled tabs cannot be selected.
function DisabledTabs(): Node {
  return (
    <Tabs>
      <Tab>Account</Tab>
      <Tab disabled={true}>Profile</Tab>
      <Tab>Settings</Tab>
    </Tabs>
  );
}
--- Controlled Tabs
value follows state; menu lists every tab for crowded bars.
function ControlledTabs(): Node {
  const current = signal('overview');
  return (
    <View className="flex-col gap-2 w-96">
      <Tabs variant="underline" menu={true} value={current.get()} onChange={(value) => { current.set(value); }}>
        <Tab value="overview">Overview</Tab>
        <Tab value="analytics">Analytics</Tab>
        <Tab value="reports">Reports</Tab>
        <Tab value="notifications">Notifications</Tab>
      </Tabs>
      <View className="flex-row gap-2">
        <Button size="small" onPress={() => { current.set('reports'); }}>Show reports</Button>
        <Text variant="muted">Selected: {current}</Text>
      </View>
    </View>
  );
}
=== Tooltip | tooltip | Overlays
Hints shown while the pointer rests on an element: text, a keyboard shortcut or rich content. Many controls also take a tooltip prop directly.
--- Basic Tooltip with Text
The children are the trigger.
function BasicTooltip(): Node {
  return (
    <Tooltip text="This is a helpful tooltip">
      <View className="rounded-md border border-border px-3 py-2"><Text>Hover me</Text></View>
    </Tooltip>
  );
}
--- Button with Tooltip
Buttons and other controls take a tooltip prop.
function ButtonTooltip(): Node {
  return <Button tooltip="Save the current document">Save</Button>;
}
--- Tooltip with Keybinding
keys shows a shortcut; command shows the keys bound to a Shortcut command.
function KeyTooltips(): Node {
  return (
    <View className="flex-row gap-3">
      <Shortcut command="tooltip-delete" keys="secondary-backspace" />
      <Tooltip text="Delete item" keys="delete"><Button>Delete</Button></Tooltip>
      <Tooltip text="Delete everything" command="tooltip-delete"><Button variant="danger">Delete all</Button></Tooltip>
    </View>
  );
}
--- Custom Element Tooltip
TooltipContent holds icons and styled text.
function RichTooltip(): Node {
  return (
    <Tooltip>
      <View className="rounded-md border border-border px-3 py-2"><Text>Hover for rich content</Text></View>
      <TooltipContent>
        <View className="flex-row items-center gap-1">
          <Icon name="info" />
          <Text variant="muted">Muted Text</Text>
          <Text className="text-destructive">Danger Text</Text>
          <Icon name="arrow-up" />
        </View>
      </TooltipContent>
    </Tooltip>
  );
}
--- Complex Tooltip Content
Several lines with a heading.
function ComplexTooltip(): Node {
  return (
    <Tooltip>
      <View className="rounded-md border border-border px-3 py-2"><Text>Hover for details</Text></View>
      <TooltipContent>
        <View className="flex-row items-center gap-1"><Icon name="user" /><Text className="text-sm font-semibold">User Information</Text></View>
        <Text className="text-xs" variant="muted">Last login: 2 hours ago</Text>
        <Text className="text-xs text-emerald-500">Status: Active</Text>
      </TooltipContent>
    </Tooltip>
  );
}
--- Tooltips in Form Elements
Switch, Checkbox and Radio take tooltips too; tooltipDelay changes the wait.
function FormTooltips(): Node {
  return (
    <View className="flex-col gap-2">
      <Switch tooltip="Toggle push notifications on/off">Enable notifications</Switch>
      <Checkbox tooltip="Keep me logged in for 30 days" tooltipDelay={0}>Remember me</Checkbox>
      <Radio tooltip="Select this option to enable feature X">Option 1</Radio>
    </View>
  );
}
=== Popover | popover | Overlays
Floating content opened from a trigger: forms, menus or details, positioned against the trigger.
--- Basic Popover
PopoverTrigger opens PopoverContent; clicking outside closes it.
function BasicPopover(): Node {
  return (
    <Popover>
      <PopoverTrigger><Button outline={true}>Open popover</Button></PopoverTrigger>
      <PopoverContent>
        <Text className="font-semibold">Dimensions</Text>
        <Text variant="muted">Set the dimensions for the layer.</Text>
      </PopoverContent>
    </Popover>
  );
}
--- Positioning
anchor picks the popup corner aligned to the trigger; offset adds a gap; arrow points at the trigger.
function PositionedPopovers(): Node {
  return (
    <View className="flex-row gap-3">
      <Popover anchor="top-left" offset={8}>
        <PopoverTrigger><Button>Below left</Button></PopoverTrigger>
        <PopoverContent><Text>Anchored top-left</Text></PopoverContent>
      </Popover>
      <Popover anchor="top-right" arrow={true}>
        <PopoverTrigger><Button>Below right, arrow</Button></PopoverTrigger>
        <PopoverContent><Text>Anchored top-right</Text></PopoverContent>
      </Popover>
      <Popover anchor="bottom-center">
        <PopoverTrigger><Button>Above</Button></PopoverTrigger>
        <PopoverContent><Text>Anchored bottom-center</Text></PopoverContent>
      </Popover>
    </View>
  );
}
--- Right-click Trigger
mouseButton="right" opens on a secondary click.
function RightClickPopover(): Node {
  return (
    <Popover mouseButton="right">
      <PopoverTrigger><View className="rounded-lg border border-dashed border-border p-6"><Text>Right-click here</Text></View></PopoverTrigger>
      <PopoverContent><Text>Opened with the right button</Text></PopoverContent>
    </Popover>
  );
}
--- Form in a Popover
Inputs work inside; overlayClosable=false keeps it open while you click elsewhere.
function FormPopover(): Node {
  const open = signal(false);
  const width = signal('100%');
  return (
    <View className="flex-row items-center gap-3">
      <Popover open={open.get()} onOpenChange={(next) => { open.set(next); }} overlayClosable={false}>
        <PopoverTrigger><Button>Edit width</Button></PopoverTrigger>
        <PopoverContent>
          <Label>Width</Label>
          <Input value={width} onChange={(event) => { width.set(event.target.value); }} className="w-48" />
          <Button size="small" variant="primary" onPress={() => { open.set(false); }}>Done</Button>
        </PopoverContent>
      </Popover>
      <Text variant="muted">Width: {width}</Text>
    </View>
  );
}
--- Controlled Open State
open follows state; onOpenChange reports the popover's own requests.
function ControlledPopover(): Node {
  const open = signal(false);
  return (
    <View className="flex-row items-center gap-3">
      <Popover open={open.get()} onOpenChange={(next) => { open.set(next); }}>
        <PopoverTrigger><Button>Toggle</Button></PopoverTrigger>
        <PopoverContent><Text>Open is {open.get() ? 'true' : 'false'}</Text></PopoverContent>
      </Popover>
      <Button outline={true} onPress={() => { open.set(!open.get()); }}>Open from outside</Button>
    </View>
  );
}
--- Without Default Styling
appearance=false leaves the surface to your own classes.
function BarePopover(): Node {
  return (
    <Popover appearance={false}>
      <PopoverTrigger><Button>Custom surface</Button></PopoverTrigger>
      <PopoverContent><View className="rounded-xl bg-primary p-4"><Text className="text-primary-foreground">Styled by utilities</Text></View></PopoverContent>
    </Popover>
  );
}
=== Accordion | accordion | Layout
Stacked sections that expand and collapse, one at a time or several together.
--- Basic Accordion
AccordionTrigger headings with AccordionContent bodies; one item open at a time.
function BasicAccordion(): Node {
  return (
    <Accordion defaultValue="item-1" className="w-full">
      <AccordionItem value="item-1">
        <AccordionTrigger><Text>Is it accessible?</Text></AccordionTrigger>
        <AccordionContent><Text>Yes. It follows the WAI-ARIA disclosure pattern.</Text></AccordionContent>
      </AccordionItem>
      <AccordionItem value="item-2">
        <AccordionTrigger><Text>Is it styled?</Text></AccordionTrigger>
        <AccordionContent><Text>Yes. It matches the active theme.</Text></AccordionContent>
      </AccordionItem>
      <AccordionItem value="item-3">
        <AccordionTrigger><Text>Is it animated?</Text></AccordionTrigger>
        <AccordionContent><Text>Opening and closing are immediate and keyboard friendly.</Text></AccordionContent>
      </AccordionItem>
    </Accordion>
  );
}
--- Multiple Open
multiple keeps several items open.
function MultipleAccordion(): Node {
  return (
    <Accordion multiple={true} defaultValue={'shipping\nreturns'} className="w-full">
      <AccordionItem value="shipping" title="Shipping"><Text>Orders ship in 2-3 days.</Text></AccordionItem>
      <AccordionItem value="returns" title="Returns"><Text>Return within 30 days.</Text></AccordionItem>
      <AccordionItem value="warranty" title="Warranty"><Text>One year on parts and labor.</Text></AccordionItem>
    </Accordion>
  );
}
--- Icons and Disabled Items
An icon before the heading; a disabled item cannot open.
function IconAccordion(): Node {
  return (
    <Accordion className="w-full">
      <AccordionItem icon="user" title="Profile"><Text>Name, photo and bio.</Text></AccordionItem>
      <AccordionItem icon="bell" title="Notifications"><Text>Email and push settings.</Text></AccordionItem>
      <AccordionItem icon="lock" title="Security (admins only)" disabled={true}><Text>Hidden</Text></AccordionItem>
    </Accordion>
  );
}
--- Sizes and Borders
small without a border, and large.
function AccordionSizes(): Node {
  return (
    <View className="flex-col gap-4 w-full">
      <Accordion size="small" bordered={false}>
        <AccordionItem title="Small, borderless"><Text>Compact content.</Text></AccordionItem>
      </Accordion>
      <Accordion size="large">
        <AccordionItem title="Large"><Text>Roomier content.</Text></AccordionItem>
      </Accordion>
    </View>
  );
}
--- Custom Styling
Utility classes for the heading, body and hovered heading.
function StyledAccordion(): Node {
  return (
    <Accordion className="w-full">
      <AccordionItem title="Styled section" titleClassName="font-semibold text-primary" contentClassName="bg-muted p-4" hoverClassName="bg-accent">
        <Text>Muted body with extra padding.</Text>
      </AccordionItem>
    </Accordion>
  );
}
--- Controlled
value lists the open items one per line; onChange reports each toggle.
function ControlledAccordion(): Node {
  const open = signal('a');
  return (
    <View className="flex-col gap-2 w-full">
      <Accordion multiple={true} value={open.get()} onChange={(values) => { open.set(values.join('\n')); }}>
        <AccordionItem value="a" title="Section A"><Text>A</Text></AccordionItem>
        <AccordionItem value="b" title="Section B"><Text>B</Text></AccordionItem>
      </Accordion>
      <View className="flex-row items-center gap-2">
        <Button size="small" onPress={() => { open.set('a\nb'); }}>Open all</Button>
        <Button size="small" onPress={() => { open.set(''); }}>Close all</Button>
        <Text variant="muted">Open: {open.get().split('\n').join(', ')}</Text>
      </View>
    </View>
  );
}
