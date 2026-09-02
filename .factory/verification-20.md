# Independent verification 20 — FAIL

Verified 2026-09-02 UTC from a clean checkout at candidate commit
`739d42da50fff5452ce4704a21b212fc597ebfb6` against
<https://class-capacity-truth.sociobot.in>.

## Release decision

**FAIL — the live deployment does not match the candidate.** Fresh production
evidence from `GET /health` was HTTP 200 with:

```json
{"status":"ok","build":"af3cf4d5231482116f926e141ec29441775a76b6","database":"ready"}
```

The required candidate is `739d42da50fff5452ce4704a21b212fc597ebfb6`.
Because the traffic-serving backend reports a different full build identity,
this candidate cannot be accepted, regardless of the local test results.
No product code, deployment, cloud resource, or unrelated service was changed.

## First read and demo

**PASS.** A cold visit says that the product is for “language schools and
tutoring centres,” tells the visitor to “Show the right number of class seats,”
and explains “Match booking counts to the class capacity your staff set.” The
first primary action is **Try it with sample data**, with “See three sample
classes next.” In plain words: it lets staff at small language schools and
tutoring centres keep the seats families see aligned with the class capacity;
click **Try it with sample data** first.

That one click opened `/demo?demo=1`, with the Bright Path Languages sample,
three realistic classes (open, full, cutoff), and the persistent **Demo —
sample data, nothing is saved** banner containing Reset demo and Start for
real. A normal sample booking changed two open seats to one. Full and cutoff
classes expose no booking form. Clearing the prefilled name and entering an
invalid email caused browser validation with no POST; correcting both reached
“Your sample seat is booked.”

## Claims and local quality gates

`.factory/claims.json` exists and contains 24 claims. From this clean checkout:

- `npm ci` passed (170 packages; 0 reported vulnerabilities).
- Every exact command declared in `claims.json` was run first. The browser
  claim commands, API claim commands, `npm run test:durable-restart`,
  `bash scripts/test-zero-config.sh`, and `npm run test:deployment` completed.
  The independent aggregate confirmations passed: `npm run test:e2e` reports
  `status: passed` with no failed tests; `npm test` reports 8 frontend tests,
  6 Rust unit tests, 21 API tests, and both deployment regressions all passed.
- `npm run lint` passed (TypeScript, rustfmt, and Clippy with warnings denied).
- `npm run build` passed and produced `dist/` plus the optimized API binary.
  Initial home JavaScript is 73.72 kB gzip and CSS 4.62 kB gzip.

## Live privacy, backend, accessibility, and performance checks

- Cold landing, demo, and signed-out workspace request logs contained only
  same-origin resources before an explicit sign-in or checkout action. No
  third-party font, script, tracker, or analytics request was observed.
- The live `api/demo/session` allowance is **10 requests per forwarded client
  IP**. A 65-request concurrent smoke with one `X-Forwarded-For` value saw 10
  HTTP 200 responses and 55 HTTP 429 responses. Every 429 included
  `Retry-After` (observed values 5 or 119 seconds), and the response included
  `X-RateLimit-Limit: 10` and `X-RateLimit-Remaining: 0`.
- Desktop and 390 px mobile checks had no console or page errors and no
  horizontal overflow. Keyboard tabbing begins with the skip link and all
  observed controls have a visible 3 px focus ring. Reduced-motion media was
  honoured. Axe found zero serious/critical violations on `/`, `/demo?demo=1`,
  `/app`, `/privacy`, `/terms`, and the designed 404 page.
- `/`, demo, app, privacy, and terms returned 200; an unknown route returned
  a designed HTTP 404. Responses include a restrictive response-header CSP
  with `frame-ancestors 'none'`, `nosniff`, strict referrer policy, and a
  restrictive permissions policy. Hashed JavaScript and CSS are served with
  `Cache-Control: public, max-age=31536000, immutable`.
- Fresh mobile Lighthouse: **100 performance / 100 accessibility**, LCP
  **1.2 s**, CLS **0**, and TBT **0 ms**.

The product is a web-with-backend application, not a library, CLI, or PWA;
consumer-install, service-worker update, and offline-reload checks do not
apply. There is no offline claim.

## Defects by severity

| Severity | Finding |
| --- | --- |
| P0 release blocker | Live `/health` identifies traffic-serving backend build `af3cf4d5231482116f926e141ec29441775a76b6`, not requested candidate `739d42da50fff5452ce4704a21b212fc597ebfb6`. Deploy the exact candidate and verify the full health build value before release. |
| P1 | None found in the candidate's local behavior. |
| P2 | None found. |
| P3 | None found. |

