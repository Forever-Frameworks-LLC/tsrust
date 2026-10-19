// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node } from 'std:ui';

export type KbdProps = ViewProps;
/** Displays keyboard text. It does not install a keyboard shortcut. */
export function Kbd(props: KbdProps): Node {
  const classes = 'pointer-events-none inline-flex flex-row items-center justify-center select-none bg-muted text-muted-foreground in-data-[slot=tooltip-content]:bg-background/20 in-data-[slot=tooltip-content]:text-background dark:in-data-[slot=tooltip-content]:bg-background/10 h-5 w-fit min-w-5 gap-1 rounded-sm px-1 font-sans text-xs font-medium ' + (props.className ?? '');
  return (<View {...props} as="kbd" data-slot="kbd" className={classes}/>);
}
/** Groups keycaps and separators in source order. */
export function KbdGroup(props: KbdProps): Node {
  const classes = 'inline-flex flex-row items-center gap-1 ' + (props.className ?? '');
  return (<View {...props} as="kbd" data-slot="kbd-group" className={classes}/>);
}
