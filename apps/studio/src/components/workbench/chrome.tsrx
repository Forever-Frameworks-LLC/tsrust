import { unsavedTargets } from '../../services/save-actions.tsr';
import { accessibleName, message } from '../../lib/localization.tsr';
import { View, Button, Shortcut, AppMenuBar, AppMenu, MenuItem, Signal, Node } from 'std:ui';
import { Workspace, closeFile } from '../../lib/workspace/workspace.tsr';
import { showSettings } from '../settings/settings-window.tsrx';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { canFormat } from '../../lib/workspace/formatting-actions.tsr';
import { zoomWorkspace } from '../../lib/workspace/zoom.tsr';

export function closeTab(workspace: Signal<Workspace>, recovery: Signal<Recovery>, path: string): void {
  const next = workspace.get();

  match (closeFile(next, path)) {
    Ok(_) => {},
    Err(message) => {
      next.error = clone(message);
    }
  }

  publishWorkspace(next, workspace, recovery);
}

export function Chrome(workspace: Signal<Workspace>, sidebar: Signal<boolean>, mode: Signal<string>, font: Signal<int>, panel: Signal<string>, recovery: Signal<Recovery>, settings: Signal<StudioConfig>): Node {
  // macOS retains its traffic-light controls in the leading 80 logical pixels.
  return (
    <View className="relative shrink-0 h-9 bg-muted border-b border-border" windowDragRegion={true}>
      <AppMenuBar>
        <AppMenu label={message("Application")}>
          <MenuItem command="open-settings" label={message("Settings")} />
        </AppMenu>
        <AppMenu label={message("File")}>
          <MenuItem command="save-all" label={message("Save All")} disabled={unsavedTargets(workspace.get()).length === 0} />
          <MenuItem command="close-tab" label={message("Close Editor")} />
        </AppMenu>
        <AppMenu label={message("Edit")}>
          <MenuItem command="format-document" label={message("Format document")} disabled={!canFormat(workspace.get().active)} />
        </AppMenu>
        <AppMenu label={message("Project")}>
          <MenuItem command="format-project" label={message("Format project")} />
          <MenuItem command="check-project" label={message("Check project")} />
        </AppMenu>
      </AppMenuBar>
      <View className="absolute left-20 top-0 bottom-0 flex-row items-center">
        <Button
          icon="panel-left"
          variant="ghost"
          size="small"
          aria-label={accessibleName("Toggle sidebar")}
          tooltip={message("Toggle sidebar")}
          tooltipCommand="toggle-sidebar"
          onPress={() => {
            sidebar.set(!sidebar.get());
          }}
        />
      </View>
      <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-2">
        <Button
          icon="settings"
          variant="ghost"
          size="small"
          aria-label={accessibleName("Settings")}
          tooltip={message("Settings")}
          onPress={() => {
            showSettings(font, settings, workspace);
          }}
        />
      </View>
      <Shortcut
        command="toggle-sidebar"
        keys="secondary-b"
        onTrigger={() => {
          sidebar.set(!sidebar.get());
        }}
      />
      <Shortcut
        command="find-file"
        keys="secondary-p"
        onTrigger={() => {
          mode.set('filesearch');
          sidebar.set(true);
        }}
      />
      <Shortcut
        command="zoom-in"
        keys="secondary-="
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, 1);
        }}
      />
      <Shortcut
        command="zoom-in-plus"
        keys="secondary-+"
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, 1);
        }}
      />
      <Shortcut
        command="zoom-out"
        keys="secondary--"
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, -1);
        }}
      />
      <Shortcut
        command="zoom-reset"
        keys="secondary-0"
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, 0, true);
        }}
      />
      <Shortcut
        command="toggle-terminal"
        keys="secondary-j"
        onTrigger={() => {
          panel.set(panel.get() === 'terminal' ? '' : 'terminal');
        }}
      />
      <Shortcut
        command="open-settings"
        keys="secondary-,"
        onTrigger={() => {
          showSettings(font, settings, workspace);
        }}
      />
      <Shortcut
        command="search-workspace"
        keys="secondary-shift-f"
        onTrigger={() => {
          mode.set('search');
          sidebar.set(true);
        }}
      />
      <Shortcut
        command="close-tab"
        keys="secondary-w"
        onTrigger={() => {
          closeTab(workspace, recovery, workspace.get().active);
        }}
      />
    </View>
  );
}
