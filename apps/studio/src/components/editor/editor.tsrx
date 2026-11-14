import { Sender } from 'std:futures';
import { extname } from 'std:path';
import {
  View,
  Button,
  CodeEditor,
  Text,
  translateWith,
  Markdown,
  DockPanel,
  Node,
  Signal,
  computed,
  signal
} from 'std:ui';

import { LanguageReferences } from '../../lib/language/references.tsr';
import { showNativeLanguageReferences } from '../../services/lsp/references.tsr';
import { navigateNativeLanguage } from '../../lib/language/navigation.tsr';
import { accessibleName, message } from '../../lib/localization.tsr';
import { updateDraft, publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { QueuedFormatting, enqueueFormatting } from '../../lib/workspace/formatting-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, OpenFile, selectFile } from '../../lib/workspace/workspace.tsr';
import { LanguageState, requestNativeLanguage, cancelNativeLanguage } from '../../services/lsp/controller.tsr';
import { ModelRequest } from '../../lib/ai/protocol.tsr';
import { AiState, requestInline, cancelInline } from '../../services/ai/actions.tsr';
import { LanguageMessage } from '../../services/lsp/session.tsr';
import { ImagePreview } from './image-preview.tsrx';

import { toggleLineBreakpoint } from '../tasks/debug-console.tsrx';
import { SourceBreakpoint } from '../../lib/workspace/workspace.tsr';

export function FilePanel(
  file: ReadOnly<OpenFile>,
  workspace: Signal<Workspace>,
  font: Signal<int>,
  recovery: Signal<Recovery>,
  focusQueue: Signal<QueuedFormatting[]>,
  language: Signal<LanguageState>,
  messages: Sender<LanguageMessage>,
  ai: Signal<AiState>,
  model: Sender<ModelRequest>,
  references: Signal<LanguageReferences>,
  panel: Signal<string>,
  breakpoints: Signal<SourceBreakpoint[]>,
  debugging: Signal<boolean>
): Node {
  const path = clone(file.document.path);
  const markerPath = clone(path);
  const clickPath = clone(path);
  const markerLines = computed<string>(() => breakpoints.get().filter((point) => point.path === markerPath).map((point) => point.line.toString()).join(','));
  const editPath = clone(path);
  const caretPath = clone(path);
  const languagePath = clone(path);
  const referencePath = clone(path);
  const languageRequests = messages.share();
  const languageCancels = messages.share();
  const inlineCancels = model.share();
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

  if (file.kind === 'binary' || file.kind === 'bytes') {
    const previewText = clone(file.binaryPreview ?? '');
    const count = file.byteSize ?? 0;
    const shown = file.previewBytes ?? 0;
    return (
      <DockPanel value={clone(path)} title={label.get()} padded={false} splitControls={true} closable={closable.get()}>
        <View id={clone(file.document.session)} className="flex-col size-full min-w-0 min-h-0">
          <Text role="status" className={'px-2 py-1 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
            {translateWith('binaryPreviewSize', {
              count, shown
            })}
          </Text>
          <CodeEditor value={previewText} readOnly={true} softWrap={false} appearance={false} bordered={false} className={editorClass.get()} accessibilityLabel={message("Hexadecimal preview")} />
        </View>
      </DockPanel>
    );
  }

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
            initialCaret={file.caret ?? 0}
            onCaretChange={(offset) => {
              const next = workspace.get();
              const index = next.files.findIndex((entry) => entry.document.path === caretPath);

              if (index >= 0 && (next.files[index].caret ?? 0) !== offset) {
                next.files[index].caret = offset;
                publishWorkspace(next, workspace, recovery);
              }
            }}
            breakpointLines={markerLines.get()}
            onBreakpointClick={(line) => {
              if (!debugging.get()) toggleLineBreakpoint(workspace, recovery, breakpoints, clickPath, line);
            }}
            onInlineCompletion={(request) => {
              requestInline(ai, model, request);
            }}
            onLanguageRequest={(request) => {
              requestNativeLanguage(
                workspace,
                language,
                languageRequests,
                languagePath,
                request,
                panel,
                references
              );
            }}
            onLanguageReferences={(locations) => {
              showNativeLanguageReferences(
                workspace,
                recovery,
                references,
                panel,
                locations,
                referencePath
              );
            }}
            onLanguageNavigate={(location) => {
              navigateNativeLanguage(workspace, recovery, location);
            }}
            onLanguageCancel={(request) => {
              cancelInline(ai, inlineCancels, request);
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
            softWrap={false}
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
