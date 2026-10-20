// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node, ReactiveValue, useProp } from 'std:ui';

export interface AlertProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {
  /** Live palette. Defaults to default; also updates data-variant and descendant styles. */
  variant?: ReactiveValue<'default' | 'destructive'>;
}
/** Contextual live content. Defaults to role=alert; use status for polite updates. Replacement renderers receive styled props once. */
export function Alert(props: AlertProps): Node {
  const variant = useProp<'default' | 'destructive'>(props.variant ?? 'default');
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'alert';
  const role = clone(props.role) ?? 'alert';
  props.variant = undefined;
  props.className = undefined;
  return (
    <View {...props} role={role} data-slot={slot} data-variant={variant.get()}
      className={'group/alert relative w-full grid grid-cols-1 gap-0.5 rounded-lg border px-2.5 py-2 text-left text-sm has-data-[slot=alert-action]:pr-18 bg-card ' + (variant.get() === 'destructive' ? 'text-destructive ' : 'text-card-foreground ') + classes.get()} />
  );
}
/** Title styling without an implicit heading. Forward a Text heading when appropriate. */
export function AlertTitle(props: ViewProps<ReactiveValue<string>, ReactiveValue<string>>): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'alert-title';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'font-medium ' + classes.get()} />);
}
/** Muted description; destructive alerts use the destructive token at 90% alpha. Caller classes override defaults. */
export function AlertDescription(props: ViewProps<ReactiveValue<string>, ReactiveValue<string>>): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'alert-description';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'text-sm text-muted-foreground group-data-[variant=destructive]/alert:text-destructive/90 ' + classes.get()} />);
}
/** Direct child action slot, positioned 8px from the top/right at default spacing. Supply your own Button or Link. */
export function AlertAction(props: ViewProps<ReactiveValue<string>, ReactiveValue<string>>): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'alert-action';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'absolute top-2 right-2 ' + classes.get()} />);
}
