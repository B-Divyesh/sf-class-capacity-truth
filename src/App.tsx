import { foundationTitle } from "./lib/routes";

export function App() {
  document.title = foundationTitle;

  return (
    <>
      <a className="skip-link" href="#main">
        Skip to main content
      </a>
      <header className="site-header">
        <a className="wordmark" href="/" aria-label="Class Capacity Truth home">
          <span aria-hidden="true" className="wordmark-mark">
            ●━●━●
          </span>
          Class Capacity Truth
        </a>
      </header>
      <main id="main" className="foundation">
        <section aria-labelledby="foundation-title">
          <p className="eyebrow">Planning foundation</p>
          <h1 id="foundation-title" tabIndex={-1}>
            The capacity product is planned.
          </h1>
          <p className="foundation-copy">
            This build skeleton gives the M1 builder typed tooling, test coverage, route titles,
            visual tokens, and an abacus component contract.
          </p>
          <div className="rail" aria-label="Design system seed: six available seat beads">
            <span className="rail-line" aria-hidden="true" />
            {Array.from({ length: 6 }, (_, index) => (
              <span className="bead" aria-hidden="true" key={index} />
            ))}
          </div>
        </section>
      </main>
      <footer className="site-footer">
        <span>Capacity truth for small schools.</span>
        <span>Built by Param Factory · foundation 0.1.0</span>
      </footer>
    </>
  );
}
