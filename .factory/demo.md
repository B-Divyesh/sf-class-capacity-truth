# Demo sandbox contract

Status: preserved by the repair and covered by four demo claim-tagged browser tests. Eight additional claims cover the real school path.

## Entry and sample

The production demo URL is https://class-capacity-truth.sociobot.in/demo?demo=1; locally it is /demo?demo=1. The landing action reaches it in one click. The service creates a random anonymous workspace marked demo, scoped to an HttpOnly cookie, and expires it after 24 hours. It is never an organisation record usable by signed-in routes.

The fictional Bright Path Languages sample has:

- “Level check: upper primary”, capacity 8, six confirmed places, open for booking;
- “Friday conversation group”, capacity 6, six confirmed places, clearly full;
- “Saturday assessment”, capacity 10, booking cutoff already passed.

The prefilled name and example.org address are fictional. Entered name and email values are validated in memory but replaced with a non-identifying marker in the demo database. No production contact, calendar connection, email delivery, AI request, or billing record is read or written in demo mode.

## Reset and isolation

The persistent “Demo — sample data, nothing is saved” banner provides **Reset demo** and **Start for real**. Reset destroys only the current demo workspace then seeds a new one. Start for real discards it and goes to the public landing or sign-in path. No action can convert a demo booking into a real booking.

The browser stores no demo data. Backend calls derive the demo tenant only from its signed HttpOnly cookie and reject an organisation parameter. Cleanup runs after 24 hours. API creation/reset is rate limited. Browser claim tests begin in fresh contexts and confirm that a reset or new context sees seed counts, not another context's booking.

## Verification mapping

sample-booking-updates-seats books the available sample class and observes the count change from two to one. full-class-blocks-booking and cutoff-blocks-booking prove the other samples prevent confirmation. demo-reset-isolated books one context, resets it, and starts another to prove the seed and isolation contract. All demo requests must be same-origin during the flow.
