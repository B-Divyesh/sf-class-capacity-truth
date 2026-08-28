# Component inventory

This inventory is a build contract, not a promise that all components exist in the foundation. M1 implements items marked M1 and records accessibility behaviour in component tests. Later builders extend shared primitives rather than recreating spacing, state, or copy.

| Component | Milestone | States and requirements |
| --- | --- | --- |
| App shell, skip link, header, footer | M1 | Current link, compact menu, keyboard close, route announcement. |
| Wordmark / rail mark | M1 | Home link, text fallback, decorative SVG hidden from screen readers. |
| Primary, secondary, quiet, danger button | M1 | Default, hover, focus, pressed, disabled, pending with retained label. |
| Text field and field group | M1 | Label, help, required, invalid, valid, disabled; no placeholder-only labels. |
| Capacity rail | M1 | Open, held, confirmed, full, cutoff, discrepancy; text count and legend. |
| Seat bead | M1 | Status shape/label, selected, reduced-motion static update. |
| Availability summary | M1 | Available, full, cutoff, unknown/conservative; never color-only. |
| Booking form and confirmation | M1 | Empty, validation error, pending, success, full race response, offline retry. |
| Demo banner | M1 | Persistent sample notice, reset pending/success, start-real link. |
| Status callout | M1 | Information, success, warning, error; role and live-announcement policy. |
| Empty / loading / error state | M1 | Named next action, skeleton respects motion setting, retry does not duplicate writes. |
| Toast region | M1 | Polite success; assertive error only; close button and no focus theft. |
| Dialog / confirmation sheet | M2 | Native focus containment, labelled title, Escape, opener focus restore. |
| Class session row/editor | M2 | Draft, valid, invalid, saving, archived, read-only entitlement. |
| Role and billing panel | M2 | Owner/operator/viewer permission, checkout return, grace state. |
| Reconciliation item | M3 | Synced, changed, conflict, loading, retry, resolved with audit reason. |
| Waitlist queue and offer card | M3 | Consent, empty, sent, accepted, expired, skipped, delivery error. |
| Activity/audit table | M4 | Loading, filters, no results, text alternative, mobile record layout. |
| Export/delete flow | M4 | Scope confirmation, pending, download, irreversible-delete warning, completion. |
