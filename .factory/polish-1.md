# Polish round 1 — review mapping

Reviewed repair commit: `f1b5523b527df482d9bd93ad719466e05f56ffc0`. Every
identifier below maps to a completed change and an executable check.

| Finding | Change made | Evidence |
| --- | --- | --- |
| F-1-1 | **Start for real** now calls the isolated-demo leave endpoint and routes to `/app`; the rendered real-start heading receives focus. | `Start for real discards demo data…` Playwright test at 390px |
| F-1-2 | Replaced the unsupported room-list promise with the actual capacity-setting workflow. | landing copy audit; `npm run test:e2e` |
| F-1-3 | Kept the disclosed USD 99/month price and made its claim fixture include USD 99, 9900 cents, and monthly billing before checking checkout navigation. | `@claim:school-plan-price` |
| F-1-4 | Split sign-in from server roles; added a CIAM browser claim checking host, client, callback, code response, and S256 PKCE. | `@claim:entra-sign-in`; `claim_staff_roles_enforce_owner_actions` |
| F-1-5 | Replaced implementation-language copy with the observable booking/count outcome. | `@claim:sample-booking-updates-seats` |
| F-1-6 | Removed the untestable student-record boundary claim and directs schools to use their existing records. | landing copy audit |
| F-1-7 | Added shared 404 shell, legal links, favicon, canonical, description, OG, and Twitter metadata. | standalone 404 Playwright/mobile test |
| F-1-8 | Added every stable workspace route to `sitemap.xml`. | route sweep Playwright test |
| F-1-9 | Corrected the calendar-poll claim location to workspace and README. | claims JSON inspection; API claim |
| F-1-10 | Changed mobile button text to Open menu / Close menu. | mobile-menu keyboard Playwright test |
| F-1-11 | Replaced the decorative preview label with Live seat preview. | landing copy audit |
| F-1-12 | Replaced the decorative boundary label with Class capacity only. | landing copy audit |
| F-1-13 | Standardised landing capacity language on seat. | landing copy audit |
| F-1-14 | Standardised the sample name on Level check: upper primary. | demo and booking browser claims |
| F-1-15 | Changed README heading to the job, Keep class seat counts accurate. | README copy audit |
| F-1-16 | Replaced capacity ledger jargon with a plain purpose sentence. | README copy audit |
| F-1-17 | Defined iCalendar in the opening workflow sentence. | README copy audit |
| F-1-18 | Replaced approved channel with the school’s usual email or messaging service. | README and workspace browser flow |
| F-1-19 | Replaced SMTP relay jargon with configured email delivery in README. | `claim_configured_smtp_queues_an_encrypted_offer` |
| F-1-20 | Replaced tenant terminology with a school Sociobot Microsoft account. | `@claim:entra-sign-in` |
| F-1-21 | Replaced stable-Entra wording with plain Microsoft sign-in role wording. | `claim_staff_roles_enforce_owner_actions` |
| F-1-22 | Replaced durable receipt jargon with saved receipt and delivery state. | `@claim:released-seat-delivery` |
| F-1-23 | Replaced CSPRNG wording with secure random key. | `bash scripts/test-zero-config.sh` |
| F-1-24 | Split the acronym-heavy architecture sentence into plain sentences. | README copy audit |
| F-1-25 | Split the replica/mount/rate-limit sentence. | README copy audit |
| F-1-26 | Split metrics access details into two plain sentences. | metrics API regression |
| F-1-27 | Split metrics content details into plain sentences. | metrics API regression |
| F-1-28 | Removed the unlisted counter-lifetime promise. | README copy audit |
| F-1-29 | Split the release verification statement into two plain sentences. | README copy audit |
| F-1-30 | Removed the untested non-root statement from public documentation. | README copy audit |
| F-1-31 | Removed the untested shared-infrastructure access-boundary statement. | README copy audit |
| F-1-32 | Removed the standalone untested fictional-sample statement. | demo claims and README copy audit |
| F-1-33 | Replaced factual art provenance footer copy with Abacus visual system. | footer browser sweep |
| F-1-34 | Standardised visitor-facing wording on guardian. | landing, README, and workspace browser sweep |

## Verification

Local evidence: `npm test`, `npm run test:e2e -- --retries=0 --reporter=line`,
`npm run test:api`, `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`,
`npm run test:deployment`, and `npm run build`. The e2e run includes axe serious/
critical checks, privacy request recording, mobile reflow, reduced motion, route
focus, demo isolation, 404, all declared browser claims, and the new CIAM claim.

Live URLs and screenshots are added after container deployment: `/`, `/demo?demo=1`,
`/app`, `/privacy`, `/terms`, and an unknown 404 URL.
