import {
  View,
  Text,
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  Diff,
  Signal,
  Node,
  t,
  translate
} from 'std:ui';
import { ProposedEdit, apply, propose } from '../../lib/workspace/documents.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { EditBatch } from '../../lib/ai/edits.tsr';
import { AiState } from '../../services/ai/actions.tsr';
import { message } from '../../lib/localization.tsr';

// Publish once after every edit has matched its captured document.
function applyBatch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, batch: ReadOnly<EditBatch>): EditBatch | undefined {
  const next = workspace.get();
  const files: ProposedEdit[] = [];

  for (const edit of batch.files) {
    const path = clone(edit.path);
    const index = next.files.findIndex((file) => file.document.path === path && file.kind === 'text');

    if (index < 0 || !apply(next.files[index].document, edit)) return undefined;
    files.push(propose(next.files[index].document, clone(edit.before)));
  }

  publishWorkspace(next, workspace, recovery);
  return {
    files
  };
}

export function undoChatEdit(state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>): void {
  const chat = state.get();
  const undo = chat.undoEdit;

  if (undo === undefined) return;

  if (applyBatch(workspace, recovery, undo) === undefined) {
    const next = workspace.get();
    next.error = translate('A target draft changed or closed. The chat edit could not be undone.');
    workspace.set(next);
  }

  chat.undoEdit = undefined;
  state.set(chat);
}

export function reviewChatEdit(state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, target: ReadOnly<EditBatch>, accept: boolean): void {
  const chat = state.get();
  const proposal = chat.review;

  if (proposal === undefined || JSON.stringify(proposal) !== JSON.stringify(clone(target))) return;

  if (accept) {
    const inverse = applyBatch(workspace, recovery, proposal);

    if (inverse !== undefined) chat.undoEdit = inverse;
    else chat.notice = translate('The file changed. Request a new edit proposal.');
  }

  chat.review = undefined;
  state.set(chat);
}

function Review(target: EditBatch, state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const accept = clone(target);
  const reject = clone(target);
  const dismiss = clone(target);
  return (
    <Dialog
      open={true}
      title={message('Review chat edit')}
      width={1000}
      centered={true}
      onOpenChange={(open) => {
        if (!open) reviewChatEdit(state, workspace, recovery, dismiss, false);
      }}
    >
      <DialogContent>
        <View className="flex-col h-[480px] min-h-0 overflow-y-auto">
          {...clone(target.files).map((file: ReadOnly<ProposedEdit>) => (
            <View className="flex-col h-[460px] shrink-0">
              <Text>
                {clone(file.path)}
              </Text>
              <Diff originalText={clone(file.before)} modifiedText={clone(file.after)} filePath={clone(file.path)} mode="split" fontSize={font.get()} className="grow min-h-0" />
            </View>
          ))}
        </View>
        <DialogFooter>
          <Button
            variant="ghost"
            onPress={() => {
              reviewChatEdit(state, workspace, recovery, reject, false);
            }}
          >
            {t('Reject')}
          </Button>
          <Button
            onPress={() => {
              reviewChatEdit(state, workspace, recovery, accept, true);
            }}
          >
            {t('Apply to draft')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function pendingReview(state: ReadOnly<AiState>): EditBatch[] {
  const review = state.review;
  return review === undefined ? [] : [clone(review)];
}

export function ChatEditReview(state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  return (
    <View className="absolute size-0">
      {...pendingReview(state.get()).map((item: ReadOnly<EditBatch>) => Review(clone(item), state, workspace, recovery, font))}
    </View>
  );
}
