# Class Capacity Truth — visual thesis

## Direction: modular classroom abacus

Capacity is not an abstract dashboard number here. It is a row of physical places that move from open to held to confirmed. The product uses a modular classroom abacus: dark chalkboard rails, warm paper panels, and clear numbered beads. A full class looks visibly full before anyone reads a table. A calendar difference is a bead slightly off its rail with a written explanation; a released seat is a bead returning to the open end.

This is a deliberately calm, useful classroom object, not a generic software dashboard or decorative hero gradient. It fits small school operations because their job is counting places under time pressure, and it gives that count a stable, inspectable physical grammar. It is familiar without becoming childish or pretending to be a full school information system.

## Stack decision

The UI uses React 19 + Vite + strict TypeScript because booking, capacity, reconciliation, and staff dashboard flows have interdependent form and asynchronous state. It uses hand-authored CSS and tokenized primitives rather than a component framework so the abacus grammar remains specific to this product. The planned API is Rust/Axum/SQLx/PostgreSQL, with a SQLite zero-config local fallback. There are no hosted font or script dependencies.

## Tokens

src/styles/tokens.css is the implementation source of truth. Builders add a token rather than scattering a new color or spacing value.

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| --color-canvas | #E8EFE8 | #081C1E | Page field, like a quiet classroom wall/chalkboard. |
| --color-surface | #FFF9EC | #133237 | Paper panel and app frame. |
| --color-surface-raised | #FFFFFF | #1B4145 | Elevated form and popover. |
| --color-ink | #102A2E | #FFF9EC | Primary text and rail outlines. |
| --color-muted | #425A5D | #C5D3CC | Supporting text, never status alone. |
| --color-rail | #23535A | #7FB2A4 | Abacus rail, dividers, focus context. |
| --color-open | #E5B23F | #F2C85B | Available-seat bead; used with ink text. |
| --color-held | #4C8B83 | #84C8B6 | Held/processing seat and information state. |
| --color-confirmed | #1E5960 | #75B4AE | Confirmed seat, paired with an icon/text label. |
| --color-warning | #9B5C00 | #F2C85B | Cutoff or needs-attention callout. |
| --color-danger | #AE3030 | #FF8C8C | Full, error, and destructive actions. |
| --color-focus | #075A73 | #F2C85B | High-contrast focus ring. |
| --color-chalkboard | #102A2E | #081C1E | Stable dark material behind class rails and teaching steps. |
| --color-chalk | #FFF9EC | #FFF9EC | Text on the chalkboard material. |
| --color-chalk-muted | #C5D3CC | #C5D3CC | Supporting text on the chalkboard material. |
| --color-confirmed-contrast | #FFF9EC | #102A2E | Number color inside a confirmed bead. |

Ink on surface, muted on surface, and white on danger are selected for at least 4.5:1 contrast. Open, held, and confirmed always carry a label and/or shape as well as color.

Typography is intentionally local: **Georgia, Times New Roman, serif** for schoolbook-style display headings, and **system-ui, -apple-system, Segoe UI, sans-serif** for operational copy and tabular figures. No font file or network request is needed; this is a self-contained readable utility. Interface body text is 16px minimum and 17px on the primary booking form. The scale is 14, 16, 20, 25, 31, 39px. Numbers use tabular figures.

Spacing follows an 8px rhythm: 4, 8, 12, 16, 24, 32, 48, 64, 96. Layout measure is 68ch maximum for explanation and 1180px for app content. Corners are practical rather than pill-like: 6px controls, 12px panels, 20px large scenes. Rails are 4px; only seat beads use a 50% radius. Shadows are short dark-green paper lifts, never glass blur.

## Interaction and motion

A successful reservation slides a single bead 12px along its rail, then the count changes; a released seat returns from the confirmed group to the open group. Reconciliation selects the mismatched bead/row before revealing its explanation. These transitions are 180–220ms, transform/opacity only, and there is no looping animation. Hover gives controls a 1px lift; press returns them to paper. Reduced motion makes changes instant or opacity-only. Focus is a high-contrast square ring with offset so it remains clear against a round bead.

## Original asset provenance

No generated image is used in this foundation. The planned hero, favicon, social card, rails, beads, and status marks are hand-authored SVG/CSS geometry in this repository: straight rails, circles, numeric labels, and paper blocks. They contain no third-party artwork, logo, font, or raster source. Provenance: Param Factory hand-made, 2026-08-28, MIT repository licence.

If a later milestone generates an image, it records the model, prompt, date, output filename, licence, responsive formats, review notes, and meaningful alt text in this document before shipping. No image contains text a user must read.

## Key screens in words

1. **Landing and demo entry:** A paper invitation sits over two dark rails. One rail shows open sample seats and another confirmed seats. The plain sentence and Try it with sample data action sit on paper, never on busy art. The next section moves directly into a working sample instead of feature-card wallpaper.
2. **Parent booking:** The class name, time, cutoff, and plain “X seats open” appear above one large rail. A labelled number of seats control is followed by guardian details and one verb-first action. Full, cutoff, waitlist, and confirmation states replace the form in place without losing context.
3. **Staff capacity board:** Sessions are rail rows grouped by day, with text total at left and an action at right. A discrepancy has an offset bead plus “Needs a calendar check,” never an unexplained red dot. Independent sessions can use paper panels; the dashboard is not a grid of identical cards.
4. **Calendar checks:** A narrow source column connects a calendar event to the affected session rail. The central explanation says what changed, when it was last checked, and the exact safe action. An empty state says the seat ledger and selected calendar agree.
5. **Waitlist and settings:** The queue is a numbered rail with consent and offer expiry text. An offer moves to a sent pocket, with resend/skip controls that require a reason. Billing, data, and roles use quieter paper forms and never compete with capacity rails.

## Responsive and accessibility rules

At 390px, navigation collapses to an accessible labelled menu, staff rails stack description over count, tables become labelled records, and the booking action remains visible without covering content. Large rails may scroll horizontally only inside a labelled region; the same seat count and status stays in normal text above it. Targets are at least 44×44px with 8px separation. Text zooms to 200% without clipped controls.

Every screen has one h1, landmarks, a skip link, linked labels, native controls where possible, announced results/errors, and focus restoration after a route or dialog. Empty, loading, error, offline, full, expired, and permission states say what happened and the next action. Product copy consistently uses guardian, class, seat, waitlist, and calendar check.
