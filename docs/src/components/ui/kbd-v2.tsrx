// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node, ReactiveValue, useProp } from 'std:ui';

export interface KbdProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {}

export interface KbdGroupProps extends KbdProps {}

/** Displays keyboard text without registering a shortcut. Caller classes remain reactive. */
export function Kbd(props: KbdProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'kbd';
  props.className = undefined;
  return (<View {...props} as="kbd" data-slot={slot} className={"pointer-events-none inline-flex flex-row items-center justify-center select-none bg-muted text-muted-foreground in-data-[slot=tooltip-content]:bg-background/20 in-data-[slot=tooltip-content]:text-background dark:in-data-[slot=tooltip-content]:bg-background/10 h-5 w-fit min-w-5 gap-1 rounded-sm px-1 font-sans text-xs font-medium [&_svg:not([class*='size-'])]:size-3 " + classes.get()}/>);
}

/** Groups keycaps and separators in source order; it does not install keyboard handlers. */
export function KbdGroup(props: KbdGroupProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'kbd-group';
  props.className = undefined;
  return (<View {...props} as="kbd" data-slot={slot} className={'inline-flex flex-row items-center gap-1 ' + classes.get()}/>);
}
