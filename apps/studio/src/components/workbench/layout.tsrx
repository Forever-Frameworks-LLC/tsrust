import { View, Resizable, ResizablePanel, Node, Computed } from 'std:ui';

// Native ordering keeps owned editor subtrees mounted when regions move.
export function Region(content: Node, panel: Node, position: Computed<string>, visible: Computed<boolean>, size: number, minimum: number): Node {
  return <Resizable vertical={position.get() === 'bottom'} reverse={position.get() === 'left'} className="size-full min-w-0 min-h-0">
    <ResizablePanel minSize={160}>{content}</ResizablePanel>
    <ResizablePanel size={size} minSize={minimum} maxSize={480} visible={visible.get()}>
      <View className={position.get() === 'left' ? 'flex-col size-full min-h-0 bg-muted border-r border-border' : position.get() === 'right' ? 'flex-col size-full min-h-0 bg-muted border-l border-border' : 'flex-col size-full min-h-0 bg-muted border-t border-border'}>{panel}</View>
    </ResizablePanel>
  </Resizable>;
}
