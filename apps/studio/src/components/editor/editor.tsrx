import { Sender } from 'std:futures';
import { extname } from 'std:path';
import {
  View,
  Button,
  CodeEditor,
  Markdown,
  DockPanel,
  Node,
  Signal,
  computed,
  signal
} from 'std:ui';

import { accessibleName, message } from '../../lib/localization.tsr';
import { updateDraft, publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { QueuedFormatting, enqueueFormatting } from '../../lib/workspace/formatting-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, OpenFile, selectFile } from '../../lib/workspace/workspace.tsr';
import { LanguageState, requestNativeLanguage, cancelNativeLanguage } from '../../services/lsp/controller.tsr';
import { LanguageMessage } from '../../services/lsp/session.tsr';
import { ImagePreview } from './image-preview.tsrx';

export function FilePanel(
  file: ReadOnly<OpenFile>,
  workspace: Signal<Workspace>,
  font: Signal<int>,
  recovery: Signal<Recovery>,
  focusQueue: Signal<QueuedFormatting[]>,
  language: Signal<LanguageState>,
  messages: Sender<LanguageMessage>
): Node {
  const path = clone(file.document.path);
  const editPath = clone(path);
  const languagePath = clone(path);
  const languageRequests = messages.share();
  const languageCancels = messages.share();
  const focusPath = clone(path);
  const blurPath = clone(path);
  const labelPath = clone(path);
  const name = clone(file.name);
  const label = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === labelPath);
    return name + (current !== undefined && dirty(current.document) ? ' ●' : '');
  });
  const closePath = clone(path);
  const closable = computed<boolean>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === closePath);
    return current === undefined || !dirty(current.document);
  });
  const textPath = clone(path);
  const text = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === textPath);
    return current === undefined ? '' : clone(current.document.text);
  });
  const preview = signal(false);
  const markdown = extname(path).toLowerCase() === '.md';
  const editorClass = computed<string>(() => 'grow min-h-0 font-mono text-[' + font.get().toString() + 'px]');

  if (file.kind === 'image') {
    return (
      <DockPanel value={clone(path)} title={label.get()} padded={false} splitControls={true} closable={closable.get()}>
        <View id={clone(file.document.session)} className="size-full min-w-0 min-h-0">
          {ImagePreview(file, workspace, font)}
        </View>
      </DockPanel>
    );
  }

  return (
    <DockPanel value={clone(path)} title={label.get()} padded={false} splitControls={true} closable={closable.get()}>
      <View id={clone(file.document.session)} className="flex-col size-full min-h-0">
        <View hidden={!markdown} className="flex-row justify-end shrink-0 border-b border-border px-2">
          <Button
            icon="code"
            variant="ghost"
            size="xsmall"
            selected={!preview.get()}
            tooltip={message("Markdown source")}
            aria-label={accessibleName("Markdown source")}
            onPress={() => {
              preview.set(false);
            }}
          />
          <Button
            icon="eye"
            variant="ghost"
            size="xsmall"
            selected={preview.get()}
            tooltip={message("Markdown preview")}
            aria-label={accessibleName("Markdown preview")}
            onPress={() => {
              preview.set(true);
            }}
          />
        </View>
        <View hidden={preview.get()} className="flex-col grow min-h-0">
          <CodeEditor
            id={clone(file.document.path)}
            language={extname(file.document.path).slice(1)}
            value={text.get()}
            onLanguageRequest={(request) => {
              requestNativeLanguage(workspace, language, languageRequests, languagePath, request);
            }}
            onLanguageCancel={(request) => {
              cancelNativeLanguage(language, languageCancels, request);
            }}
            onChange={(value) => {
              updateDraft(workspace, recovery, editPath, value);
            }}
            onFocus={() => {
              const next = workspace.get();

              if (next.active !== focusPath && selectFile(next, clone(focusPath))) publishWorkspace(next, workspace, recovery);
            }}
            onBlur={() => {
              const pending = focusQueue.get();

              enqueueFormatting(pending, workspace.get(), blurPath, 'focus');
              focusQueue.set(pending);
            }}
            appearance={false}
            bordered={false}
            accessibilityLabel={clone(file.document.path)}
            className={editorClass.get()}
          />
        </View>
        <View hidden={!preview.get()} className="grow min-h-0 overflow-y-auto p-4">
          <Markdown content={text.get()} scrollable={true} />
        </View>
      </View>
    </DockPanel>
  );
}
