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
import { ProposedEdit, apply } from '../../lib/workspace/documents.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { AiState } from '../../services/ai/actions.tsr';
import { message } from '../../lib/localization.tsr';

export function reviewChatEdit(state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, target: ReadOnly<ProposedEdit>, accept: boolean): void {
  const chat = state.get();
  const proposal = chat.review;

  if (proposal === undefined || proposal.session !== target.session || proposal.revision !== target.revision || proposal.path !== target.path || proposal.after !== target.after) return;

  if (accept) {
    const next = workspace.get();
    const path = clone(proposal.path);
    const index = next.files.findIndex((file) => file.document.path === path && file.kind === 'text');

    if (index < 0 || !apply(next.files[index].document, proposal)) chat.notice = translate('The file changed. Request a new edit proposal.');
    else publishWorkspace(next, workspace, recovery);
  }

  chat.review = undefined;
  state.set(chat);
}

function Review(target: ReadOnly<ProposedEdit>, state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
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
        <Text>
          {clone(target.path)}
        </Text>
        <View className="flex-col h-[480px] min-h-0">
          <Diff originalText={clone(target.before)} modifiedText={clone(target.after)} filePath={clone(target.path)} mode="split" fontSize={font.get()} className="size-full" />
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

function pendingReview(state: ReadOnly<AiState>): ProposedEdit[] {
  const review = state.review;
  return review === undefined ? [] : [clone(review)];
}

export function ChatEditReview(state: Signal<AiState>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  return (
    <View className="absolute size-0">
      {...pendingReview(state.get()).map((item: ReadOnly<ProposedEdit>) => Review(item, state, workspace, recovery, font))}
    </View>
  );
}
