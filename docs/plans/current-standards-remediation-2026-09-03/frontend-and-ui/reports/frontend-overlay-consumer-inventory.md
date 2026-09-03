# Frontend Overlay Consumer Inventory

## Scope And Method

This inventory covers the plan's bounded modal frames/feature dialogs and the
version, model-filter, and remote-download popups. The source census searched
for fixed/absolute overlay positioning, dialog/modal attributes, expansion
state, document/window input listeners, focus calls, and every planned
consumer call site.

Disposition vocabulary is `migrate`, `already safe`, `delete`, or `follow-up`.
Feature content, domain actions, and in-flow disclosures do not move into the
shared interaction Modules.

## Modal Consumers

| Consumer | Current interaction semantics | Gap / invariant | Disposition and target policy |
| --- | --- | --- | --- |
| `ConfirmationDialog` | Named/described `alertdialog`; focuses Cancel; capture-phase Escape stops the parent listener; backdrop and close controls dismiss; restores the previously focused element | No focus containment; backdrop is disabled while confirming but Escape remains active; focus/stack policy is a private duplicate | `migrate` as the representative nested-capable consumer. Preserve `alertdialog` and description, make dismissal-disabled policy apply consistently, contain focus, and restore through the shared stack. |
| `InstallDialogFrame` dialog branch | Named `dialog`; focuses Close; Escape/backdrop dismiss; restores prior focus | No containment; global listener/restoration duplicates policy; contains a nested confirmation workflow | `migrate`. The shared Module owns focus, Escape/backdrop, cleanup, and nested restoration. The page branch remains non-modal and does not use `ModalDialog`. |
| `InstallDialogFrame` page branch | In-flow page content with no dialog role or backdrop | It is intentionally not an overlay | `already safe`; preserve page semantics and do not focus the close control. |
| `ModelMetadataModalFrame` | Named modal; focuses Close; Escape/backdrop dismiss; restores prior focus | No containment and duplicated lifecycle | `migrate`; preserve feature header/refetch/content and current dismiss policy. |
| `ModelServeDialog` dialog branch | Named modal; custom hook focuses the profile selector, wraps Tab at endpoints, and closes on Escape | No restoration or backdrop policy; fixed title id; focus hook is a second incomplete lifecycle owner | `migrate`; shared Module owns generated name relationship, initial focus, containment, Escape, explicit backdrop policy, and restore. |
| `ModelServeDialog` page branch | In-flow page content | `useDialogFocusTrap` focuses the profile selector even when its `isEnabled` flag is false | `migrate` by removing modal focus machinery from page mode; otherwise preserve the page content. |
| `ModelImportDialog` | Visually modal with a visible Close control; backdrop does not dismiss | The accepted Chromium baseline observed no dialog/modal semantics, no focus entry, no Escape dismissal, and no focus restoration | `migrate`; preserve deliberate non-dismissing backdrop to avoid accidental workflow loss, add named modal semantics, initial focus, containment, Escape, and restoration. |
| `HuggingFaceAuthDialog` | Visually modal; backdrop and two close affordances dismiss | No dialog/name, focus entry/containment, Escape, or restoration; icon close is unnamed | `migrate`; preserve backdrop dismissal, add modal lifecycle, initially focus token input when present (otherwise the close/action control), and name the icon close control. |

`ConfirmationDialog` is rendered inside both `InstallDialog` and
`MigrationReportsPanel`; the install path is the required nested-modal oracle.
Only the topmost mounted modal may respond to Escape or contain focus. On close,
the stack restores to the connected opener inside the parent modal before the
parent itself can restore to its opener.

## Popup Consumers

None of the three popups is a listbox: selection changes are immediate domain
actions, and version/download surfaces contain multiple action types. Treat
them as named non-modal action dialogs rather than promising menu/listbox arrow
navigation that the product does not implement.

| Consumer | Current interaction semantics | Gap / invariant | Disposition and target policy |
| --- | --- | --- | --- |
| `VersionSelector` / trigger / dropdown | A version button toggles an absolute action panel; parent closes on outside mouse/touch and version selection; rows contain both a switch action and a default-version action | Trigger has no `aria-expanded`, `aria-controls`, or popup type; panel has no role/name; no Escape, focus entry, or focus return | `migrate` to controlled `Popover`; use a named non-modal action dialog, focus the active/first version action, close on Escape/outside/selection, and return focus to the opener. Delete the parent document listeners. |
| `ModelSearchBar` category/kind filter | Named filter button exposes only `aria-expanded`; selecting a filter closes | No relationship/popup type, role/name, Escape, outside dismissal, focus entry, or return | `migrate` to controlled `Popover`; use a named non-modal action dialog with the mode-correct label, focus the selected/first filter, and preserve selection behavior. |
| `RemoteModelListItemActions` / `RemoteModelDownloadMenu` | A download-options button (or queue-another-download action) toggles an absolute panel containing quant buttons or checkboxes plus download actions | Trigger uses `aria-pressed` instead of an expansion relationship; panel has no role/name; no Escape/outside dismissal, focus entry, or return | `migrate` to controlled `Popover`; use a model-named non-modal action dialog. The currently applicable options/queue control is the opener, and mixed checkbox/button content remains ordinary native controls rather than a false menu. |

The `Popover` Module therefore owns one controlled open-state Interface,
trigger/panel ids and `aria-haspopup="dialog"`, opener capture, initial focus,
Escape/outside dismissal, focus return, and listener cleanup. Callers own
labels, placement/style, selected item knowledge, and actions. This is the
smallest common policy; it does not add generic menu, disclosure, or selection
state.

## Delete And Exact-Write-Set Findings

- Delete `model-serve/useDialogFocusTrap.ts` after `ModelServeDialog` is the
  last migrated consumer.
- Removing that hook also removes the now-mechanism-only `dialogRef` prop from
  `model-serve/ModelServeDialogContent.tsx`. That file was absent from the
  planned write set and is added before source edits.
- The trigger relationship for the remote popup belongs in
  `RemoteModelListItemActions.tsx`; a focused test file does not exist. Add
  `RemoteModelListItemActions.test.tsx` before source edits rather than asking
  the menu-content test to prove an opener it cannot render.
- No hook, app root, `ModelManager`, or remote-list state owner must change:
  the new popover accepts controlled state and the existing close/toggle
  actions already reach each planned consumer.

## Searched Non-Members

| Surface | Disposition | Reason |
| --- | --- | --- |
| `ModelImportDropZone` | `already safe` for this invariant family | Full-screen drag feedback has no keyboard action or dialog interaction and exists only while a drag is active; it does not share modal focus/dismiss lifecycle. |
| `LinkHealthStatus` and `MigrationReportsPanel` expanded sections | `follow-up` (`FE-I12`) | They are in-flow disclosures rather than popups. Their triggers lack disclosure state/relationships, but they do not share outside-click, focus entry/return, or overlay positioning. |
| `Tooltip`, `IconButton`, and `HoldToDeleteButton` hover labels | `already safe` for this bounded overlay migration | The rendered labels are noninteractive `role="tooltip"` content with `pointer-events: none`; named buttons do not require modal/popover focus lifecycle. General tooltip activation policy is not claimed here. |
| `InstallDialogContent`, `ProgressDetailsView`, link-health details, and dependency-status transitions | `already safe` for this invariant family | AnimatePresence is used for in-flow state/content, not an independent overlay. |
| `AppSidebar` | `already safe` for this invariant family | It is an always-present named toolbar, not a popup or overlay despite owning Escape selection clearing. |

## Slice Boundaries And Evidence

1. `M2-S2`: build `ModalDialog`, prove its public lifecycle with a nested
   representative test, migrate `ConfirmationDialog`, then migrate the other
   matching modal branches and delete `useDialogFocusTrap`/stale ref plumbing.
2. `M2-S3`: build `Popover`, prove its controlled Interface, migrate the three
   popup families, and delete their local outside/focus lifecycle code.
3. `M2-S4`: run the admitted Chromium workflow for import modal behavior plus
   a nested installation-confirmation workflow and representative popup
   focus/Escape/return workflow, then reconcile FE-A4.

Focused jsdom tests remain necessary for exhaustive Tab boundaries, topmost
Escape, dismissal-disabled state, listener cleanup, controlled transitions,
and feature-domain preservation. The admitted Electron/CDP harness is the
deciding oracle for browser focus and accessibility-tree outcomes.

## Stopping Condition

Every member of the bounded modal/popup population has a disposition; searched
nearby overlays and disclosures have explicit non-member dispositions; the two
required write-set discoveries are recorded before mutation; and the shared
Modules' responsibilities are narrower than feature-domain behavior.
