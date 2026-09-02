# Copy audit

Audited 2026-09-02 for polish round 2. Counts use whitespace-separated words.
No listed sentence exceeds 22 words or uses a banned marketing word.

## Landing page

| Copy | Words | Result |
| --- | ---: | --- |
| For language schools and tutoring centres | 6 | Pass |
| Show the right number of class seats | 7 | Pass |
| Match booking counts to the class capacity your staff set. | 10 | Pass |
| Try it with sample data | 5 | Pass |
| See three sample classes next. | 5 | Pass |
| The demo stays separate and resets. | 6 | Pass; `demo-reset-isolated` |
| No advertising trackers or analytics scripts. | 6 | Pass; covered by `no-third-party-tracking` |
| The plan costs $99 per school each month. | 8 | Pass; `school-plan-price` |
| 2 seats open | 3 | Pass |
| Level check: upper primary | 4 | Pass |
| 8 seats − 6 booked = 2 open | 8 | Pass |
| Live seat preview | 3 | Pass |
| Count seats before taking a booking | 6 | Pass |
| Book a sample seat and see the open count change. | 10 | Pass; `sample-booking-updates-seats` |
| Follow one seat from open to booked | 7 | Pass |
| Compare an open class with full and closed examples. | 9 | Pass |
| Enter a sample guardian name and example.org email. | 8 | Pass |
| The class changes from two open seats to one. | 9 | Pass; `sample-booking-updates-seats` |
| Keep school records in your existing system | 7 | Pass |
| Use your existing school records for grades, attendance, tuition, and learning history. | 12 | Pass |
| Create a class, publish its booking link, compare calendar bookings, and record released-seat offers. | 14 | Pass |
| Version 0.1.0. | 2 | Pass |

Read-aloud check: “Show the right number of class seats. Match booking counts
to the class capacity your staff set. Try it with sample data.” It names the
job, audience, and first action in one breath.

## README changes

| Copy | Words | Result |
| --- | ---: | --- |
| The plan costs $99 per school each month through Sociobot checkout. | 11 | Pass; `school-plan-price` |
| Each browser gets its own temporary workspace for 24 hours. | 10 | Pass; `demo-expiry-input-disposal` |
| The demo checks each name and email, then discards both. | 10 | Pass; `demo-expiry-input-disposal` |
| These optional settings configure email delivery: SMTP_RELAY, SMTP_USERNAME, SMTP_PASSWORD, and SMTP_FROM. | 10 | Pass |
| Without them, staff can copy the saved offer URL and see that no email was sent. | 17 | Pass; `released-seat-delivery` |
| The Playwright suite starts the compiled API service with a clean temporary database and checks browser flows. | 17 | Pass |
| Run every command in .factory/claims.json to verify all claims. | 9 | Pass |
| Rust, Axum, SQLx, and SQLite store the isolated demo and school workspaces. | 12 | Pass |
| Concurrent booking requests cannot oversell a class. | 7 | Pass; `concurrent-booking-does-not-oversell` |
| The metrics response lists request counts, server errors, and response times. | 11 | Pass; `operational-metrics-no-pii` |
| Build and deploy the checked-out commit with its full commit ID. | 12 | Pass |
| The deployment command rejects unbound tags and requires /health to return that ID. | 13 | Pass |
| Public and demo pages load no third-party fonts, scripts, advertising trackers, or analytics. | 13 | Pass; `no-third-party-tracking` |
| Microsoft sign-in and Sociobot checkout open only after a staff member selects them. | 12 | Pass; `no-third-party-tracking` |

## Terminology

| Concept | Word used |
| --- | --- |
| Scheduled group offering | class |
| Unit of capacity | seat |
| Adult contact | guardian |
| Temporary evaluation data | sample data |
| Last permitted booking time | booking cutoff |
| Connected source review | calendar check |
| Released-seat invitation | offer |
