# Class Capacity Truth

Class Capacity Truth is being built for small language schools and tutoring
centres that need the seat count shown to families to match real class places.
It is a narrow capacity and waitlist layer, not a student information system.

This commit is the planning and tooling foundation. The customer product is
not implemented yet. The executable build plan is in
[.factory/plan.md](.factory/plan.md), the visual system is in
[.factory/design.md](.factory/design.md), and the researched brief is in
[.factory/brief.json](.factory/brief.json).

## Develop

Requires Node 22+ and npm 10+.

```bash
npm install
npm run dev
npm test
npm run test:e2e
npm run build # produces dist/
```

`npm run preview` serves the production build locally. The Rust API will be
added under `services/api` by the M1 builder; see the plan for its run and
deployment contract. GitHub Actions runs unit/browser tests and a build on
every push and pull request. The factory handles deployment; this repository
does not change infrastructure, DNS, or billing configuration.

## Privacy and licence

The foundation sends no analytics and loads no third-party fonts or scripts.
The planned product minimises guardian data and keeps its demo isolated; the
M1 demo contract is documented in [.factory/demo.md](.factory/demo.md).

Released source is available under the [MIT License](LICENSE).
