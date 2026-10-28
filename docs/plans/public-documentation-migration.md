# Public documentation migration map

This map tracks canonical source moves. Repository links temporarily target the
new source files while production hosting is being prepared. Once deployment is
verified, active entry points will link to the public routes. Historical evidence
and retained UI content are not automatically published.

| Previous source | Canonical source / route | Disposition |
| --- | --- | --- |
| docs/guides/toolchain-installation.md | website/src/content/docs/tools/install-and-update.md → /tools/install-and-update/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/tooling/machine-setup.md | website/src/content/docs/tools/rust-setup.md → /tools/rust-setup/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/application-projects.md | website/src/content/docs/backend/applications.md → /backend/applications/ | Moved; old copy removed; inbound Markdown links updated. |
| docs/guides/generated-api-clients.md | website/src/content/docs/backend/api-clients.md → /backend/api-clients/, /backend/authorization/, /backend/api-compatibility/ | Split by client generation, access policy and compatibility; old copy removed; inbound Markdown links updated. |
| New site overview | website/src/content/docs/index.md → / | Reviewed initial page. |
| New quickstart | website/src/content/docs/start/quickstart.md → /start/quickstart/ | Reviewed initial journey; installation and editor detail still pending. |
| New public status | website/src/content/docs/project/status.md → /project/status/ | Current product scope; internal milestone details stay in repository. |
| docs/content/getting-started/installation.md | Planned /start/installation/ | Pending retained-content consumer migration before removal. |
| docs/content/getting-started/project-structure.md | Planned /start/project-structure/ | Pending retained-content consumer migration before removal. |
| editors/vscode/README.md | Planned /start/editor/ plus short extension README | Retain package-specific entry point; move full public guidance. |
| Remaining docs/guides/ | Language/backend/library/tools destinations in the site plan | Review and map individually before moving. |

The moved guides have no authored heading IDs or linked internal anchors to
rename. External links and behavior contracts are retained. Their existing
commands have focused compiler/installer/cache evidence; migration itself requires
site rendering and repository-link checks, not another compiler matrix.
