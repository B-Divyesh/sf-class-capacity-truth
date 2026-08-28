# Handoff — venture-class-capacity-truth-plan

Completed 2026-08-28 by the planner work order.

## What was done

- Turned the researched brief into the executable venture contract in
  [.factory/plan.md](plan.md): PRD, linked evidence, Rust/Axum plus React/Vite
  architecture, tenancy/privacy model, Entra CIAM, Sociobot/Dodo subscription,
  optional gateway-only AI, jobs, limits, operations, M1–M5 delivery plan,
  exact routes/titles, claims, tests, definitions of done, and risk
  experiments.
- Recorded the product-specific modular classroom abacus visual system in
  [.factory/design.md](design.md), including contrast-aware light/dark tokens,
  type, spacing, motion, mobile/accessibility rules, key screens, and
  hand-made asset provenance.
- Added the M1 claims contract, component inventory, and isolated demo
  specification in [.factory/claims.json](claims.json),
  [.factory/component-inventory.md](component-inventory.md), and
  [.factory/demo.md](demo.md).
- Scaffolded the Vite/React strict-TypeScript foundation, design token
  implementation, route-title contract, unit/browser test harness, hand-made
  abacus metadata assets, CSP/SWA configuration, API/contracts directories,
  and GitHub Actions CI.
- Updated README and retained the existing MIT licence. No product booking,
  account, billing, or persistence flow has been built, by explicit work-order
  instruction.

## Verification

Run from the repository root:

```bash
npm ci
npm test
npm run build
npm run test:e2e
npm audit
```

This handoff verified all five commands. Results: 3 unit tests passed; the
single Playwright foundation test passed; production build completed with
59.30 kB gzip initial JavaScript and 1.37 kB gzip CSS; npm audit reported zero
vulnerabilities.

## Known gaps

- M1 is intentionally not implemented. The claims file is the M1 contract, so
  its claim-tagged Playwright tests do not exist yet and must be added with the
  actual demo; do not treat the foundation test as claim evidence.
- There is no Rust service, database, Entra callback registration, Sociobot
  billing registration, calendar OAuth client, SMTP relay, deployed URL, or
  live Lighthouse result yet.
- The current app is a semantic foundation page, not a customer landing page.
  M1 replaces it with the planned landing, demo, booking, privacy, terms, and
  404 routes while retaining the token and metadata contracts.

## Next steps

The M1 builder should read the plan, design, demo, claims, and this handoff,
then implement only M1. Start with the Axum demo service and transaction-safe
seat ledger, retain the query-string demo entry and cookie isolation, add one
Playwright test per existing claim tag, and update the plan status plus
.factory/handoff-m1.md after review/polish PASS.
