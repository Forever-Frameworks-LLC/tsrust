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
