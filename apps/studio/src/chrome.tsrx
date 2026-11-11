import { View, Button, Shortcut, Signal, Node } from 'std:ui';
import { Workspace, closeFile, moveFile } from './workspace.tsr';
import { showSettings } from './settings-window.tsrx';
import { Recovery } from './recovery.tsr';
import { saveActive } from './document-actions.tsr';

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

export function Chrome(workspace: Signal<Workspace>, sidebar: Signal<boolean>, mode: Signal<string>, font: Signal<int>, panel: Signal<string>, recovery: Signal<Recovery>): Node {
  // macOS retains its traffic-light controls in the leading 80 logical pixels.
  return <View className="relative shrink-0 h-9 bg-muted border-b border-border" windowDragRegion={true}>
    <View className="absolute left-20 top-0 bottom-0 flex-row items-center">
      <Button icon="panel-left" variant="ghost" size="small" aria-label="Toggle sidebar" tooltip="Toggle sidebar" tooltipCommand="toggle-sidebar" onPress={() => { sidebar.set(!sidebar.get()); }} />
    </View>
    <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-2">
      <Button icon="save" variant="ghost" size="small" aria-label="Save file" tooltip="Save file" tooltipCommand="save-file" onPress={() => { saveActive(workspace, recovery); }} />
      <Button icon="terminal" variant="ghost" size="small" aria-label="Toggle terminal" tooltip="Toggle terminal" onPress={() => { panel.set(panel.get() === 'terminal' ? '' : 'terminal'); }} />
      <Button icon="settings" variant="ghost" size="small" aria-label="Settings" tooltip="Settings" onPress={() => { showSettings(font); }} />
    </View>
    <Shortcut command="toggle-sidebar" keys="secondary-b" onTrigger={() => { sidebar.set(!sidebar.get()); }} />
    <Shortcut command="save-file" keys="secondary-s" onTrigger={() => { saveActive(workspace, recovery); }} />
    <Shortcut command="find-file" keys="secondary-p" onTrigger={() => { mode.set('filesearch'); sidebar.set(true); }} />
    <Shortcut command="zoom-in" keys="secondary-=" onTrigger={() => { font.set(font.get() + 1); }} />
    <Shortcut command="zoom-in-plus" keys="secondary-+" onTrigger={() => { font.set(font.get() + 1); }} />
    <Shortcut command="zoom-out" keys="secondary--" onTrigger={() => { if (font.get() > 8) font.set(font.get() - 1); }} />
    <Shortcut command="zoom-reset" keys="secondary-0" onTrigger={() => { font.set(14); }} />
    <Shortcut command="toggle-terminal" keys="secondary-j" onTrigger={() => { panel.set(panel.get() === 'terminal' ? '' : 'terminal'); }} />
    <Shortcut command="open-settings" keys="secondary-," onTrigger={() => { showSettings(font); }} />
    <Shortcut command="search-workspace" keys="secondary-shift-f" onTrigger={() => { mode.set('search'); sidebar.set(true); }} />
    <Shortcut command="close-tab" keys="secondary-w" onTrigger={() => { closeTab(workspace, workspace.get().active); }} />
    <Shortcut command="move-tab-left" keys="secondary-alt-Left" onTrigger={() => { moveTab(workspace, -1); }} />
    <Shortcut command="move-tab-right" keys="secondary-alt-Right" onTrigger={() => { moveTab(workspace, 1); }} />
  </View>;
}
