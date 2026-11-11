import { View, Button, DropdownMenu, MenuItem, MenuSeparator, Shortcut, Signal, Node, useTheme } from 'std:ui';
import { Workspace, closeFile, moveFile } from './workspace.tsr';

export function closeTab(workspace: Signal<Workspace>, path: string): void {
  const next = workspace.get();
  match (closeFile(next, path)) {
    Ok(_) => {},
    Err(message) => {
      next.error = clone(message);
    }
  }

  workspace.set(next);
}

function moveTab(workspace: Signal<Workspace>, direction: int): void {
  const next = workspace.get();
  const path = clone(next.active);
  moveFile(next, path, direction);
  workspace.set(next);
}

export function Chrome(workspace: Signal<Workspace>, sidebar: Signal<boolean>, mode: Signal<string>, font: Signal<int>): Node {
  // macOS retains its traffic-light controls in the leading 80 logical pixels.
  return <View className="relative shrink-0 h-9 bg-muted border-b border-border" windowDragRegion={true}>
    <View className="absolute left-20 top-0 bottom-0 flex-row items-center">
      <Button icon="panel-left" variant="ghost" size="small" aria-label="Toggle sidebar" tooltip="Toggle sidebar" tooltipCommand="toggle-sidebar" onPress={() => { sidebar.set(!sidebar.get()); }} />
    </View>
    <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-2">
      <Button icon="search" variant="ghost" size="small" tooltip="Find a file" tooltipCommand="find-file" onPress={() => { mode.set('search'); sidebar.set(true); }} aria-label="Find file" />
      <DropdownMenu variant="ghost" size="small" caret={false}>
        <Button icon="settings" variant="ghost" size="small" aria-label="Settings" tooltip="Settings" />
        <MenuItem label="Increase text size" onSelect={() => { font.set(font.get() + 1); }} />
        <MenuItem label="Decrease text size" onSelect={() => { if (font.get() > 8) font.set(font.get() - 1); }} />
        <MenuItem label="Reset text size" onSelect={() => { font.set(14); }} />
        <MenuSeparator />
        <MenuItem label="Dark appearance" onSelect={() => { useTheme('Studio Dark'); }} />
        <MenuItem label="Light appearance" onSelect={() => { useTheme('Studio Light'); }} />
        <MenuSeparator />
        <MenuItem label="Move active tab left" onSelect={() => { moveTab(workspace, -1); }} />
        <MenuItem label="Move active tab right" onSelect={() => { moveTab(workspace, 1); }} />
      </DropdownMenu>
    </View>
    <Shortcut command="toggle-sidebar" keys="secondary-b" onTrigger={() => { sidebar.set(!sidebar.get()); }} />
    <Shortcut command="find-file" keys="secondary-p" onTrigger={() => { mode.set('search'); sidebar.set(true); }} />
    <Shortcut command="zoom-in" keys="secondary-=" onTrigger={() => { font.set(font.get() + 1); }} />
    <Shortcut command="zoom-in-plus" keys="secondary-+" onTrigger={() => { font.set(font.get() + 1); }} />
    <Shortcut command="zoom-out" keys="secondary--" onTrigger={() => { if (font.get() > 8) font.set(font.get() - 1); }} />
    <Shortcut command="zoom-reset" keys="secondary-0" onTrigger={() => { font.set(14); }} />
    <Shortcut command="close-tab" keys="secondary-w" onTrigger={() => { closeTab(workspace, workspace.get().active); }} />
    <Shortcut command="move-tab-left" keys="secondary-alt-Left" onTrigger={() => { moveTab(workspace, -1); }} />
    <Shortcut command="move-tab-right" keys="secondary-alt-Right" onTrigger={() => { moveTab(workspace, 1); }} />
  </View>;
}
