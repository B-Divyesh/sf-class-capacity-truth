# Current handoff — M1

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
