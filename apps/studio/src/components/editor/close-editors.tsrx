import {
  View,
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  Node,
  Signal,
  t
} from 'std:ui';

import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { CloseEditorsPlan, planCloseEditors } from '../../lib/workspace/close-editors.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { invokeCloseEditorTool, CloseEditorsContext } from '../../services/close-editor-actions.tsr';
import { message } from '../../lib/localization.tsr';

export interface CloseEditorsState {
  open: boolean;
  plan: CloseEditorsPlan;
}

function applyClose(workspace: Signal<Workspace>, recovery: Signal<Recovery>, state: Signal<CloseEditorsState>, discard: boolean): void {
  const context: CloseEditorsContext = {
    workspace: workspace.handle(), recovery: recovery.handle(), allowClose: true, allowDiscard: discard
  };

  match (invokeCloseEditorTool(context, 'close_all_editors', JSON.stringify({
    plan: state.get().plan, discard
  }))) {
    Ok(_) => {
      const next = state.get();
      next.open = false;
      state.set(next);
    },
    Err(error) => {
      const next = workspace.get();
      next.error = error;
      workspace.set(next);
    }
  }
}

export function requestCloseEditors(workspace: Signal<Workspace>, recovery: Signal<Recovery>, state: Signal<CloseEditorsState>): void {
  const files = workspace.get();
  const plan = planCloseEditors(files);
  state.set({
    open: files.files.some((file) => dirty(file.document)), plan
  });

  if (!state.get().open) applyClose(workspace, recovery, state, false);
}

export function CloseEditorsDialog(workspace: Signal<Workspace>, recovery: Signal<Recovery>, state: Signal<CloseEditorsState>): Node {
  return (
    <View className="absolute size-0">
      <Dialog
        open={state.get().open}
        title={message('Close all editors?')}
        description={message('Some editors have unsaved changes. Cancel and save them first, or explicitly discard all unsaved drafts.')}
        width={520}
        onOpenChange={(open) => {
          const next = state.get();
          next.open = open;
          state.set(next);
        }}
      >
        <DialogContent>
          <DialogFooter>
            <Button
              variant="ghost"
              onPress={() => {
                const next = state.get();
                next.open = false;
                state.set(next);
              }}
            >
              {t('Cancel')}
            </Button>
            <Button
              variant="danger"
              onPress={() => {
                applyClose(workspace, recovery, state, true);
              }}
            >
              {t('Discard all and close')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </View>
  );
}
