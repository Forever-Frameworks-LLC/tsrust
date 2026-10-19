import { View, ViewProps, Node } from 'std:ui';

export type CardProps = ViewProps;
/** Editable project container. Children retain their order and ownership. */
export function Card(props: CardProps): Node {
  const classes = 'rounded-lg border border-border bg-card text-card-foreground p-4 gap-4 ' + (props.className ?? '');
  return <View {...props} className={classes} />;
}
