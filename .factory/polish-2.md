# Polish round 2 — review mapping

Repair commit: `1a8ea791b2bc536caef11473aace2cb5e1af2b44`.

All evidence commands were run from a fresh clone of this commit. Live
re-check evidence is recorded under `.factory/evidence-polish-2/live/` after
deployment; the cold landing screenshot is `home-desktop.png` and the demo
mobile screenshot is `demo-mobile.png`.

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-2-1 | README now says Playwright checks browser flows and directs maintainers to every claims command. | fresh-clone claim sweep; `README.md` |
| F-2-2 | Every public price sentence now says “$99 per school each month”; the recorded checkout fixture asserts USD 99, 9900 cents, monthly interval, and all visible pricing surfaces. | `@claim:school-plan-price`; live `/`, `/app`, `/terms` |
| F-2-3 | Removed unverified merchant, cancellation, and refund promises. Terms state only the tested checkout behavior. | `@claim:school-plan-price`; live `/terms` |
| F-2-4 | Removed unverified controller/processor role labels; privacy describes the tested booking data flow. | `school-capacity-flow`, `contact-encryption-retention`; live `/privacy` |
| F-2-5 | Replaced the relay-recipient promise with the tested encrypted queued-offer behavior. | `configured-smtp-delivery`; live `/privacy` |
| F-2-6 | Added `concurrent-booking-does-not-oversell` and tagged the two-request last-seat race. | `npm run test:api -- concurrent_booking_requests_do_not_oversell_a_class` |
| F-2-7 | Removed the unmeasured 99.9% target from README and operations UI. | full browser sweep; live `/app/operations` |
| F-2-8 | Expanded the third-party asset/tracker claim to cover fonts, scripts, advertising, and analytics. | `@claim:no-third-party-tracking` request log |
| F-2-9 | Added the exact explicit-action boundary to the same privacy claim and tests it before and after staff select sign-in/checkout. | `@claim:no-third-party-tracking` |
| F-2-10 | Removed the broad repository infrastructure-scope guarantee from public documentation. | README copy audit |
| F-2-11 | Included **Start for real** deletion in `demo-reset-isolated`; its one tagged browser flow checks reset, exit, focus, and a fresh sample. | `@claim:demo-reset-isolated`; live `/demo?demo=1` |
| F-2-12 | First screen names language schools and tutoring centres and states the capacity-setting job. | `home-desktop.png`; live `/` |
| F-2-13 | Changed the ambiguous action result to “See three sample classes next.” | `home-desktop.png`; live `/` |
| F-2-14 | Removed the decorative footer design label. | `home-desktop.png`; live `/` |
| F-2-15 | Replaced “persistent class” with the staff task, “Create a class.” | `home-desktop.png`; live `/` |
| F-2-16 | Replaced “signed, temporary workspace” with plain 24-hour workspace wording. | README copy audit |
| F-2-17 | Rewrote the demo input sentence with a complete subject and verb. | README copy audit |
| F-2-18 | Introduced the email-delivery purpose before listing optional settings. | README copy audit |
| F-2-19 | Replaced “durable, copyable” jargon with saved offer URL and no-email result. | `@claim:released-seat-delivery`; README copy audit |
| F-2-20 | Replaced the framework name in test instructions with “compiled API service.” | README copy audit |
| F-2-21 | Replaced “single-instance school ledger” with school workspaces. | README copy audit |
| F-2-22 | Replaced “Prometheus response” with “metrics response.” | `operational-metrics-no-pii`; README copy audit |
| F-2-23 | Split the deployment instruction into separate full-commit-ID and guard sentences. | README copy audit |
| F-2-24 | Replaced SHA with “full commit ID.” | README copy audit |
| F-2-25 | Replaced Entra with Microsoft sign-in and added the explicit-action claim coverage. | `@claim:no-third-party-tracking`; live `/privacy` |

## Regression coverage

`npm test`, `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`,
`npm run lint`, `npm run build`, and the full Playwright/Axe suite pass. The
fresh-clone command executes every `test` command in `.factory/claims.json`.
The browser suite includes keyboard, 390px, 200% text, dark/reduced-motion,
route/focus, real 404, demo isolation, request privacy, and Axe serious/critical
checks.
