// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node } from 'std:ui';

export interface AlertProps extends ViewProps {
  /** Creation-time palette, default default. data-variant reflects this value. */
  variant?: 'default' | 'destructive';
}
/** Contextual live content. role defaults to alert; use status for polite updates. Owns forwarded children once. */
export function Alert(props: AlertProps): Node {
  const variant = clone(props.variant) ?? 'default';
  const slot = clone(props['data-slot']) ?? 'alert';
  const role = clone(props.role) ?? 'alert';
  const classes = 'group/alert relative w-full grid grid-cols-1 gap-0.5 rounded-lg border px-2.5 py-2 text-left text-sm has-data-[slot=alert-action]:pr-18 bg-card ' + (variant === 'destructive' ? 'text-destructive ' : 'text-card-foreground ') + (clone(props.className) ?? '');
  return (<View {...props} role={role} data-slot={slot} data-variant={variant} className={classes}/>);
}
/** Title styling without an implicit heading. Pass Text as=h2/h3 when a document heading is appropriate. */
export function AlertTitle(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'alert-title';
  const classes = 'font-medium ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes}/>);
}
/** Muted description; destructive alerts use the destructive token at 90% alpha. Caller classes override defaults. */
export function AlertDescription(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'alert-description';
  const classes = 'text-sm text-muted-foreground group-data-[variant=destructive]/alert:text-destructive/90 ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes}/>);
}
/** Direct child action slot, positioned 8px from the top/right at default spacing. Supply your own Button or Link. */
export function AlertAction(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'alert-action';
  const classes = 'absolute top-2 right-2 ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes}/>);
}
