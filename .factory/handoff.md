# Current handoff — M1

## Independent QA verdict (2026-08-28): **FAIL**

Candidate `ead522ac24c02ddcfa8b3b18c680365195daa8fc` is deployed at
https://class-capacity-truth.sociobot.in and its public M1 demo/test suite
works. It is nevertheless **not releasable as the researched product**:
schools cannot yet connect calendars, configure/publish real classes, accept
real bookings, reconcile capacity, or run a waitlist/released-seat flow.
The Dockerfile also pins `rust:1.89-alpine`, contrary to the required
un-pinned `rust:1-alpine`/`rust:1-slim` contract. See
[verification.md](verification.md) for exact test evidence and all findings.

M1 shipped on 2026-08-28. The public product now includes the landing page,
isolated SQLite-backed capacity-booking demo, legal and 404 routes, complete
claim coverage, limits, accessibility checks, and a production container.

The canonical evidence, clean-clone commands, scope decision, operator actions,
and M2 checklist are in [handoff-m1.md](handoff-m1.md).

Quick verification:

```bash
npm ci
npm test
npm run test:e2e
npm run build
```

Production: https://class-capacity-truth.sociobot.in/demo?demo=1

Next milestone: M2 adds real school workspaces with Sociobot Entra CIAM,
PostgreSQL tenant isolation, class publishing, and the registered Sociobot/Dodo
subscription. The demo remains isolated and all M1 claims must continue to pass.
