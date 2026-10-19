// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ElementProps, Node } from 'std:ui';

export interface SeparatorProps extends ElementProps {
 /** Defaults to horizontal. Orientation selects metadata and line dimensions. */
 orientation?: 'horizontal' | 'vertical';
  className?: string;
}
/** A nonfocusable accessible section divider. It accepts no children. */
export function Separator(props: SeparatorProps): Node {
  const orientation = clone(props.orientation) ?? 'horizontal';
  const classes = 'shrink-0 bg-border data-horizontal:h-px data-horizontal:w-full data-vertical:w-px data-vertical:self-stretch ' + (props.className ?? '');
  return (<View {...props} data-slot="separator" role="separator" aria-orientation={orientation} className={classes}/>);
}
