<!-- tsrust-docs
{"id":"component-card","group":"components","order":20,"status":"partial","targets":["web","native"],"component":"card","examples":["card-layout","card-project"]}
-->
# Card

The current Card ports the portable content parts of the pinned Nova surface. Image edge variants, inline-size containment and container queries remain unsupported.

## Install and import

Run `tsrust ui add card --offline`. Import the exports from your configured local
component directory. The installed source belongs to the project; theme tokens and
caller classes customize the same component on web and native.

## Composition

Card exports `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardAction`, `CardContent` and `CardFooter`. These compose owned subtrees in source order.

The header responds to action/description slots. CardContent inherits scoped card spacing. CardFooter adds a muted surface, top border and lower corners. `size` selects `default` or `sm`; omission uses `default`.

## Headings and naming

CardTitle is a layout container. Put a semantic heading inside it when the surrounding document needs a heading. CardDescription is visible text; it does not automatically label an unrelated form control.

## Customization

The local source exposes shared utilities and `data-slot` metadata. Caller classes override default properties. `data-size` takes precedence over the convenience `size` prop. Unsupported utilities throw rather than silently disappearing.
