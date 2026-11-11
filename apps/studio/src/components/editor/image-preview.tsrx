import {
  View,
  Text,
  Button,
  Image,
  Node,
  Signal,
  computed,
  t,
  tWith
} from 'std:ui';

import { accessibleName, message } from '../../lib/localization.tsr';
import { Workspace, OpenFile } from '../../lib/workspace/workspace.tsr';
import { ImageViewContext, ImageViewTarget, setImageZoom, imageZoomStep } from '../../services/image-actions.tsr';

function changeZoom(workspace: Signal<Workspace>, target: ReadOnly<ImageViewTarget>, zoom: number): void {
  const context: ImageViewContext = {
    workspace, allowViewChanges: true
  };
  unwrap(setImageZoom(context, target, zoom));
}

function reportZoom(workspace: Signal<Workspace>, target: ReadOnly<ImageViewTarget>, zoom: number): void {
  const next = workspace.get();
  const index = next.files.findIndex((file) => file.document.path === target.path && file.document.session === target.session);

  if (index < 0 || next.files[index].imageEffectiveZoom === zoom) return;
  next.files[index].imageEffectiveZoom = zoom;
  workspace.set(next);
}

export function ImagePreview(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, font: Signal<int>): Node {
  const path = clone(file.document.path);
  const target: ImageViewTarget = {
    path: clone(path), session: clone(file.document.session)
  };
  const effectivePath = clone(path);
  const effective = computed<number>(() => workspace.get().files.find((file) => file.document.path === effectivePath)?.imageEffectiveZoom ?? 100);
  const zoomPath = clone(path);
  const zoom = computed<number>(() => workspace.get().files.find((file) => file.document.path === zoomPath)?.imageZoom ?? 0);
  const scroll = computed<string>(() => JSON.stringify(workspace.get().files.find((file) => file.document.path === path)?.imageScroll ?? {
    x: 0, y: 0, request: 0
  }));
  const zoomOutTarget = clone(target);
  const zoomInTarget = clone(target);
  const fitTarget = clone(target);
  const actualTarget = clone(target);
  const toolbar = computed<string>(() => 'flex-row items-center justify-end gap-1 px-2 h-8 shrink-0 border-b border-border text-[' + font.get().toString() + 'px]');
  const label = computed<string>(() => Math.round(effective.get()).toString() + '%');
  return (
    <View className="flex-col size-full min-w-0 min-h-0">
      <View className={toolbar.get()}>
        <Button
          icon="minus"
          variant="ghost"
          size="xsmall"
          tooltip={message('Zoom out image')}
          aria-label={accessibleName('Zoom out image')}
          disabled={effective.get() <= 1}
          onPress={() => {
            changeZoom(workspace, zoomOutTarget, imageZoomStep(effective.get(), -1));
          }}
        />
        <Text>
          {label.get()}
        </Text>
        <Button
          icon="plus"
          variant="ghost"
          size="xsmall"
          tooltip={message('Zoom in image')}
          aria-label={accessibleName('Zoom in image')}
          disabled={effective.get() >= 3200}
          onPress={() => {
            changeZoom(workspace, zoomInTarget, imageZoomStep(effective.get(), 1));
          }}
        />
        <Button
          icon="scan"
          variant="ghost"
          size="xsmall"
          selected={zoom.get() === 0}
          tooltip={message('Fit image to window')}
          aria-label={accessibleName('Fit image to window')}
          onPress={() => {
            changeZoom(workspace, fitTarget, 0);
          }}
        />
        <Button
          variant="ghost"
          size="xsmall"
          selected={zoom.get() === 100}
          tooltip={message('Actual image size')}
          aria-label={accessibleName('Actual image size')}
          onPress={() => {
            changeZoom(workspace, actualTarget, 100);
          }}
        >
          <Text className={'text-[' + font.get().toString() + 'px]'}>
            {t('100%')}
          </Text>
        </Button>
      </View>
      <View className="grow min-w-0 min-h-0">
        <Image
          src={clone(file.document.path)}
          zoom={zoom.get()}
          scrollTo={scroll.get()}
          onZoomChange={(value) => {
            reportZoom(workspace, target, value);
          }}
          alt={tWith('Unable to decode {name}', {
            name: clone(file.name)
          })}
          className="size-full"
        />
      </View>
    </View>
  );
}
