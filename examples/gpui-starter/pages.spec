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
        <GroupButton label="Option 1" selected={selected.get().includes('0')} />
        <GroupButton label="Option 2" selected={selected.get().includes('1')} />
        <GroupButton label="Option 3" selected={selected.get().includes('2')} />
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
A labelled toggle.
function BasicToggle(): Node {
  const on = signal(false);
  return <Toggle label="Bold" checked={on.get()} onChange={(checked) => { on.set(checked); }} />;
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
      <Toggle label="XS" size="xsmall" />
      <Toggle label="Small" size="small" />
      <Toggle label="Medium" />
      <Toggle label="Large" size="large" />
    </View>
  );
}
--- Disabled State
A disabled toggle keeps its state but cannot change.
function DisabledToggle(): Node {
  return (
    <View className="flex-row gap-3">
      <Toggle label="Off" disabled={true} />
      <Toggle label="On" checked={true} disabled={true} />
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
        <Toggle label="List" checked={view.get() === 0} onChange={(on) => { if (on) { view.set(0); } }} />
        <Toggle label="Grid" checked={view.get() === 1} onChange={(on) => { if (on) { view.set(1); } }} />
      </ToggleGroup>
      <ToggleGroup segmented={true} size="small">
        <Toggle label="Top" />
        <Toggle label="Bottom" />
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
      <Kbd keys="secondary-k" />
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
        <MenuItem label="Copy" onSelect={() => { last.set('copy'); }} />
        <MenuItem label="Paste" onSelect={() => { last.set('paste'); }} />
        <MenuItem label="Delete" onSelect={() => { last.set('delete'); }} />
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
      <MenuItem label="Rename" onSelect={() => { last.set('rename'); }} />
      <MenuItem label="Duplicate" onSelect={() => { last.set('duplicate'); }} />
    </ContextMenu>
  );
}
--- Custom Trigger and Anchor
A Button child is the trigger with all its options; anchor picks the menu corner aligned to it.
function TriggerAndAnchor(): Node {
  return (
    <DropdownMenu anchor="top-right" menuLabel="More actions">
      <Button icon="ellipsis" variant="ghost" />
      <MenuItem label="Settings" icon="settings" />
      <MenuItem label="Sign out" icon="log-out" />
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
        <MenuItem label="Word wrap" checked={wrap.get()} onSelect={() => { wrap.set(!wrap.get()); }} />
        <MenuItem label="Minimap" icon="map" disabled={true} />
      </DropdownMenu>
      <DropdownMenu label="Checks on the right" checkSide="right">
        <MenuItem label="Word wrap" checked={wrap.get()} onSelect={() => { wrap.set(!wrap.get()); }} />
      </DropdownMenu>
    </View>
  );
}
--- Labels, Separators and Links
Labels head groups of entries; separators divide them; a link entry opens a URL.
function LabelsAndLinks(): Node {
  return (
    <DropdownMenu label="Help">
      <MenuLabel text="Documentation" />
      <MenuItem label="GPUI Kit" href="https://gpui-kit.com" />
      <MenuItem label="Tsrust" href="https://github.com/Forever-Frameworks-LLC/tsrust" />
      <MenuSeparator />
      <MenuItem label="About" icon="info" />
    </DropdownMenu>
  );
}
--- Custom Element
A MenuItem with children shows them instead of a label.
function CustomEntry(): Node {
  return (
    <DropdownMenu label="Account">
      <MenuItem><View className="flex-col"><Text className="font-semibold">Ada Lovelace</Text><Text className="text-muted-foreground">ada@example.com</Text></View></MenuItem>
      <MenuSeparator />
      <MenuItem label="Profile" icon="user" />
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
        <MenuItem label="Save all" command="menu-save" />
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
      <MenuItem label="Copy link" icon="link" />
      <Submenu label="Send to" icon="send">
        <MenuItem label="Email" />
        <Submenu label="Chat">
          <MenuItem label="Team" />
          <MenuItem label="Direct message" />
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
      <MenuItem label="Inter" />
      <MenuItem label="Roboto" />
      <MenuItem label="Source Sans" />
      <MenuItem label="Fira Sans" />
      <MenuItem label="IBM Plex Sans" />
      <MenuItem label="Noto Sans" />
      <MenuItem label="Open Sans" />
      <MenuItem label="Lato" />
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
Any theme color name or hex value.
function BadgeColors(): Node {
  return (
    <View className="flex-row items-center gap-6">
      <Badge count={3} color="blue"><Avatar name="Ada" /></Badge>
      <Badge icon="star" color="yellow"><Avatar name="Grace" /></Badge>
      <Badge dot={true} color="green"><Icon name="bell" size="lg" /></Badge>
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
Exact fill, text and border colors.
function CustomTag(): Node {
  return <Tag color="#1f6feb" foreground="#ffffff" borderColor="#1a5fc8">Custom Color</Tag>;
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
Plain label text.
function BasicLabel(): Node {
  return <Label text="This is a label" />;
}
--- Label with Secondary Text
Muted text after the label.
function SecondaryLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label text="Company Address" secondary="(optional)" />
      <Label text="Email Address" secondary="(required)" />
    </View>
  );
}
--- Text Alignment
Alignment utilities on a full-width label.
function AlignedLabels(): Node {
  return (
    <View className="flex-col gap-2 w-80">
      <Label text="Text align left" className="w-full" />
      <Label text="Text align center" className="w-full text-center" />
      <Label text="Text align right" className="w-full text-right" />
    </View>
  );
}
--- Text Highlighting
Highlights every match, or only a match at the start.
function HighlightedLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label text="Hello World Hello" highlight="Hello" />
      <Label text="Hello World" highlight="Hello" highlightPrefix={true} />
      <Label text="Company Name" secondary="(optional)" highlight="Company" />
    </View>
  );
}
--- Color and Styling
Text utilities style the label.
function StyledLabels(): Node {
  return (
    <View className="flex-col gap-2">
      <Label text="Color Label" className="text-primary" />
      <Label text="Font Size Label" className="text-xl font-semibold" />
    </View>
  );
}
--- Masked Labels
Masked text draws as bullets.
function MaskedLabel(): Node {
  const masked = signal(true);
  return (
    <View className="flex-row items-center gap-3">
      <Label text="9,182.1 USD" masked={masked.get()} className="text-2xl" />
      <Button size="small" onPress={() => { masked.set(!masked.get()); }}>{masked.get() ? 'Show' : 'Hide'}</Button>
    </View>
  );
}
--- Multi-line Text
Labels wrap inside a narrow width.
function WrappedLabel(): Node {
  return <Label text="Label should support text wrap in default, if the text is too long, it should wrap to the next line." className="w-[200px]" />;
}
--- Different Sizes
Text size utilities.
function LabelSizes(): Node {
  return (
    <View className="flex-col gap-1">
      <Label text="Extra Large" className="text-2xl" />
      <Label text="Large" className="text-xl" />
      <Label text="Medium" />
      <Label text="Small" className="text-sm" />
      <Label text="Extra Small" className="text-xs" />
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
      <Kbd keys="cmd-shift-p" />
      <Kbd keys="cmd-t" />
      <Kbd keys="escape" />
      <Kbd keys="enter" />
      <Kbd keys="backspace" />
    </View>
  );
}
--- Multiple Modifiers
Complex combinations.
function ModifierKeys(): Node {
  return (
    <View className="flex-row gap-2">
      <Kbd keys="cmd-ctrl-shift-a" />
      <Kbd keys="cmd-alt-backspace" />
      <Kbd keys="ctrl-alt-shift-a" />
    </View>
  );
}
--- Arrow Keys and Function Keys
Named keys.
function NamedKeys(): Node {
  return (
    <View className="flex-row flex-wrap gap-2">
      <Kbd keys="left" />
      <Kbd keys="right" />
      <Kbd keys="up" />
      <Kbd keys="down" />
      <Kbd keys="f12" />
      <Kbd keys="secondary-f12" />
      <Kbd keys="pageup" />
      <Kbd keys="pagedown" />
    </View>
  );
}
--- Without Visual Styling
Plain draws only the key text.
function PlainKbd(): Node {
  return <Kbd keys="cmd-s" plain={true} />;
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
      <View className="flex-row items-center gap-2"><Text>Open command palette:</Text><Kbd keys="cmd-shift-p" /></View>
      <View className="flex-row items-center gap-2"><Text>Save file:</Text><Kbd keys="cmd-s" /></View>
      <View className="flex-row items-center gap-2"><Text>Find in files:</Text><Kbd keys="cmd-shift-f" /></View>
    </View>
  );
}
--- Inline Documentation
Keys inside a sentence.
function InlineKbd(): Node {
  return (
    <View className="flex-row items-center gap-1">
      <Text>Press</Text><Kbd keys="escape" /><Text>to cancel or</Text><Kbd keys="enter" /><Text>to confirm.</Text>
    </View>
  );
}
--- Custom Styling
Utility classes restyle the key cap.
function StyledKbd(): Node {
  return <Kbd keys="cmd-k" className="text-primary border-primary bg-primary/10" />;
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
      <Switch label="Loading" checked={loading.get()} onChange={(on) => { loading.set(on); }} />
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
Theme color names or hex.
function ColoredSpinners(): Node {
  return (
    <View className="flex-row gap-4">
      <Spinner color="blue" />
      <Spinner color="green" />
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
  return <Alert message="This is a simple alert message." />;
}
--- Alert with Title
A title above the message.
function TitledAlert(): Node {
  return <Alert title="Heads up!" message="You can add components to your app using the cli." />;
}
--- Alert Variants
Default, info, success, warning and error.
function AlertVariants(): Node {
  return (
    <View className="flex-col gap-3 w-full">
      <Alert variant="default" message="This is a default alert." />
      <Alert variant="info" message="This is an info alert." />
      <Alert variant="success" message="Your changes have been saved." />
      <Alert variant="warning" message="Please review your settings." />
      <Alert variant="error" message="Something went wrong." />
    </View>
  );
}
--- Alert Sizes
xsmall to large.
function AlertSizes(): Node {
  return (
    <View className="flex-col gap-3 w-full">
      <Alert size="xsmall" message="Extra small alert" />
      <Alert size="small" message="Small alert" />
      <Alert message="Medium alert" />
      <Alert size="large" message="Large alert" />
    </View>
  );
}
--- Closable Alerts
onClose shows a close button.
function ClosableAlert(): Node {
  const open = signal(true);
  return (
    <View className="flex-col gap-3 w-full">
      <Alert variant="info" message="This alert can be closed." visible={open.get()} onClose={() => { open.set(false); }} />
      <Button size="small" onPress={() => { open.set(true); }}>Reset</Button>
    </View>
  );
}
--- Banner Mode
Full width, without rounding.
function BannerAlert(): Node {
  return <Alert variant="info" banner={true} message="This is a banner alert that spans the full width." />;
}
--- Custom Icons
Replace the variant icon.
function IconAlert(): Node {
  return <Alert variant="info" icon="bell" message="You have new notifications." />;
}
--- Conditional Visibility
visible hides the alert without removing it.
function ConditionalAlert(): Node {
  const show = signal(true);
  return (
    <View className="flex-col gap-3 w-full">
      <Switch label="Show alert" checked={show.get()} onChange={(on) => { show.set(on); }} />
      <Alert variant="warning" message="Shown while the switch is on." visible={show.get()} />
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
Text color utilities tint the icon.
function ColoredIcons(): Node {
  return (
    <View className="flex-row gap-3">
      <Icon name="heart" className="text-destructive" />
      <Icon name="circle-check" className="text-primary" />
      <Icon name="info" className="text-muted-foreground" />
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
  return <ShimmerText text="Thinking..." />;
}
--- Duration
Milliseconds per sweep.
function ShimmerDuration(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText text="Fast shimmer" duration={800} />
      <ShimmerText text="Slow shimmer" duration={3000} />
    </View>
  );
}
--- Highlight Color
Any theme color or hex.
function ShimmerColor(): Node {
  return <ShimmerText text="Generating response" highlightColor="blue" />;
}
--- Spread
Highlight half-width as a fraction of the text.
function ShimmerSpread(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText text="Narrow highlight" spread={0.1} />
      <ShimmerText text="Wide highlight" spread={0.5} />
    </View>
  );
}
--- Direction and Play-once
Reverse the sweep or play it once.
function ShimmerDirection(): Node {
  return (
    <View className="flex-col gap-2">
      <ShimmerText text="Right to left" reverse={true} />
      <ShimmerText text="Plays once" once={true} />
    </View>
  );
}
--- Styling
Text utilities style the shimmer.
function StyledShimmer(): Node {
  return <ShimmerText text="Searching the web" className="text-xl font-semibold" />;
}
=== Empty | empty | Feedback
Empty states with media, a title, a description and actions.
--- Basic Usage
An icon, title, description and actions.
function BasicEmpty(): Node {
  return (
    <Empty icon="folder" title="No projects yet" description="Create your first project to get started.">
      <View className="flex-row gap-2">
        <Button variant="primary">Create project</Button>
        <Button outline={true}>Import</Button>
      </View>
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
      <Switch label="Email" checked={true} />
      <Switch label="Push" />
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
      <Checkbox label="Subscribe to updates" />
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
Controlled: onChange receives the requested state.
function BasicCheckbox(): Node {
  const checked = signal(false);
  return <Checkbox label="Accept terms and conditions" checked={checked.get()} onChange={(on) => { checked.set(on); }} />;
}
--- Different Sizes
xsmall to large.
function CheckboxSizes(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox label="Extra Small" size="xsmall" />
      <Checkbox label="Small" size="small" />
      <Checkbox label="Medium" />
      <Checkbox label="Large" size="large" />
    </View>
  );
}
--- Disabled State
Disabled checkboxes ignore input.
function DisabledCheckbox(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox label="Disabled checkbox" disabled={true} />
      <Checkbox label="Disabled checked" checked={true} disabled={true} />
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
      <Checkbox label="Sync" tooltip="Sync across devices" tooltipDelay={300} />
      <Checkbox label="Dark mode" role="switch" />
    </View>
  );
}
--- Custom Tab Order
tabIndex orders focus; tabStop skips a control.
function TabOrderCheckbox(): Node {
  return (
    <View className="flex-col gap-2">
      <Checkbox label="Second" tabIndex={2} />
      <Checkbox label="First" tabIndex={1} />
      <Checkbox label="Skipped" tabStop={false} />
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
  return <Switch label="Enable notifications" checked={on.get()} onChange={(value) => { on.set(value); }} />;
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
      <Switch size="small" checked={true} />
      <Switch checked={true} />
    </View>
  );
}
--- Disabled State
Disabled switches keep their state.
function DisabledSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch label="Disabled off" disabled={true} />
      <Switch label="Disabled on" checked={true} disabled={true} />
    </View>
  );
}
--- Custom Color
The track color when on; disabled draws it at half opacity.
function ColoredSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch checked={true} color="success" />
      <Switch checked={true} color="danger" />
      <Switch checked={true} color="success" disabled={true} />
    </View>
  );
}
--- With Tooltip
A tooltip on hover.
function TooltipSwitch(): Node {
  return <Switch label="Auto-save" tooltip="Save changes automatically" />;
}
--- Keyboard Focus
tabIndex orders focus; tabStop skips it.
function FocusSwitch(): Node {
  return (
    <View className="flex-row items-center gap-4">
      <Switch label="Focusable" tabIndex={1} />
      <Switch label="Skipped" tabStop={false} />
    </View>
  );
}
=== Radio | radio | Forms
One choice from a set, alone or in a RadioGroup.
--- Basic Radio Button
A standalone radio.
function BasicRadio(): Node {
  const checked = signal(false);
  return <Radio label="Option 1" checked={checked.get()} onChange={(on) => { checked.set(on); }} />;
}
--- Radio Group
The group tracks the selected index.
function BasicRadioGroup(): Node {
  const selected = signal(0);
  return (
    <RadioGroup selected={selected.get()} onChange={(index) => { selected.set(index); }}>
      <Radio label="Option 1" />
      <Radio label="Option 2" />
      <Radio label="Option 3" />
    </RadioGroup>
  );
}
--- Different Sizes
xsmall to large.
function RadioSizes(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio label="Small" size="xsmall" />
      <Radio label="Medium" />
      <Radio label="Large" size="large" />
    </View>
  );
}
--- Disabled State
Disabled radios ignore input.
function DisabledRadio(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio label="Disabled" disabled={true} />
      <Radio label="Disabled checked" checked={true} disabled={true} />
    </View>
  );
}
--- Tooltip and Tab Order
Tooltips, accessibility names and focus order.
function TooltipRadio(): Node {
  return (
    <View className="flex-col gap-2">
      <Radio label="Express" tooltip="Arrives tomorrow" tabIndex={1} />
      <Radio label="Standard" accessibilityLabel="Standard shipping" tabStop={false} />
    </View>
  );
}
--- Vertical Layout
Options stacked vertically.
function VerticalRadioGroup(): Node {
  const selected = signal(1);
  return (
    <RadioGroup vertical={true} selected={selected.get()} onChange={(index) => { selected.set(index); }}>
      <Radio label="Small" />
      <Radio label="Medium" />
      <Radio label="Large" />
    </RadioGroup>
  );
}
--- Disabled Radio Group
A disabled group disables every option.
function DisabledRadioGroup(): Node {
  return (
    <RadioGroup vertical={true} selected={0} disabled={true}>
      <Radio label="Option 1" />
      <Radio label="Option 2" />
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
      <Rating size="xsmall" value={3} />
      <Rating size="small" value={3} />
      <Rating value={3} />
      <Rating size="large" value={3} />
    </View>
  );
}
--- Custom Color
Any theme color or hex.
function ColoredRating(): Node {
  return <Rating value={4} color="danger" />;
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
