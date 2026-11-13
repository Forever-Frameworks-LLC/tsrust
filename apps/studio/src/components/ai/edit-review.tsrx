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
import { EditBatch, togglePassage } from '../../lib/ai/edits.tsr';
import { AiState } from '../../services/ai/actions.tsr';
import { message } from '../../lib/localization.tsr';

// Publish once after every edit has matched its captured document.
function applyBatch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, batch: ReadOnly<EditBatch>): EditBatch | undefined {
  const next = workspace.get();
  const files: ProposedEdit[] = [];

  for (const edit of batch.files) {
    if (edit.after === edit.before) continue;
    const path = clone(edit.path);
    const index = next.files.findIndex((file) => file.document.path === path && file.kind === 'text');

    if (index < 0 || !apply(next.files[index].document, edit)) return undefined;
    files.push(propose(next.files[index].document, clone(edit.before)));
  }

  if (files.length === 0) return undefined;
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

function PassageControls(file: ReadOnly<ProposedEdit>, state: Signal<AiState>, multipleFiles: boolean): Node[] {
  const controls: Node[] = [];
  const passages = clone(file.passages);

  if (passages === undefined || (!multipleFiles && passages.length < 2)) return controls;

  for (let index = 0; index < passages.length; index += 1) {
    const passage = passages[index];
    const line = file.before.slice(0, passage.start).split('\n').length;
    const label = translate(passage.included ? 'Exclude change' : 'Include change') + ' · ' + translate('Line') + ' ' + line.toString();
    const path = clone(file.path);
    const captured = clone(file);
    const position = index;
    controls.push(
      <Button
        variant="ghost"
        onPress={() => {
          const chat = state.get();
          const review = chat.review;

          if (review === undefined) return;
          const target = review.files.findIndex((item) => item.path === path);

          if (target < 0 || JSON.stringify(review.files[target]) !== JSON.stringify(captured)) return;
          togglePassage(review.files[target], position);
          chat.review = review;
          state.set(chat);
        }}
      >
        {label}
      </Button>
    );
  }

  return controls;
}

function Review(target: EditBatch, state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const multipleFiles = target.files.length > 1;
  const empty = target.files.every((file) => file.before === file.after);
  const rows: Node[] = [];

  for (const file of target.files) {
    const controls = PassageControls(file, state, multipleFiles);
    rows.push(
      <View className="flex-col h-[460px] shrink-0">
        <Text>
          {clone(file.path)}
        </Text>
        <Diff originalText={clone(file.before)} modifiedText={clone(file.after)} filePath={clone(file.path)} mode="split" fontSize={font.get()} className="grow min-h-0" />
        {...controls}
      </View>
    );
  }

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
          {...rows}
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
            disabled={empty}
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
