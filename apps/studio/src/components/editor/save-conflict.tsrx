import { View, Text, Button, Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter, Diff, Node, Signal, t } from 'std:ui';
import { Workspace, OpenFile } from '../../lib/workspace/workspace.tsr';
import { SaveConflict } from '../../lib/workspace/documents.tsr';
import { DocumentActionContext, resolveDocumentConflict } from '../../services/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { message } from '../../lib/localization.tsr';

function conflictAction(workspace: Signal<Workspace>, recovery: Signal<Recovery>, target: ReadOnly<SaveConflict>, action: ReadOnly<string>): void {
  const context: DocumentActionContext = {
    workspace, recovery, allowDiscard: action === 'reload', allowOverwrite: action === 'overwrite'
  };
  resolveDocumentConflict(context, target, action);
}

function ConflictDialog(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const target = clone(file.document.saveConflict);

  if (target === undefined) return (
    <View />
  );
  const closing: SaveConflict = clone(target);
  const cancel = clone(target);
  const reload = clone(target);
  const overwrite = clone(target);
  const disk = clone(target.diskText);
  const draft = clone(file.document.text);
  const path = clone(file.document.path);
  return (
    <Dialog
      open={true}
      title={message('File changed on disk')}
      width={1200}
      centered={true}
      overlayClosable={false}
      onOpenChange={(open) => {
        if (!open) conflictAction(workspace, recovery, closing, 'keep-editing');
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {t('File changed on disk')}
          </DialogTitle>
          <DialogDescription>
            {clone(path)}
          </DialogDescription>
        </DialogHeader>
        <Text>
          {t('Another process changed the saved file. Your unsaved draft is safe. Compare both versions, then choose which to keep.')}
        </Text>
        <View className="flex-row">
          <View className="w-1/2">
            <Text>
              {t('File on disk')}
            </Text>
          </View>
          <View className="w-1/2">
            <Text>
              {t('Your unsaved draft')}
            </Text>
          </View>
        </View>
        <View className="flex-col h-[480px] min-h-0">
          <Diff originalText={disk} modifiedText={draft} filePath={path} mode="split" fontSize={font.get()} softWrap={true} headerVisible={false} contextLines={3} changeBackground={true} inlineUnit="word" className="size-full" />
        </View>
        <Text className="text-destructive">
          {workspace.get().error}
        </Text>
        <DialogFooter>
          <Button
            variant="ghost"
            onPress={() => {
              conflictAction(workspace, recovery, cancel, 'keep-editing');
            }}
          >
            {t('Keep editing')}
          </Button>
          <Button
            variant="secondary"
            onPress={() => {
              conflictAction(workspace, recovery, reload, 'reload');
            }}
          >
            {t('Discard draft and reload')}
          </Button>
          <Button
            variant="danger"
            onPress={() => {
              conflictAction(workspace, recovery, overwrite, 'overwrite');
            }}
          >
            {t('Overwrite with my draft')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function SaveConflicts(workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  return (
    <View className="absolute size-0">
      {...workspace.get().files.filter((file) => file.document.saveConflict !== undefined).slice(0, 1).map((file: ReadOnly<OpenFile>) => ConflictDialog(file, workspace, recovery, font))}
    </View>
  );
}
