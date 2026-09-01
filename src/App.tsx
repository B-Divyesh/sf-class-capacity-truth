import { useEffect, useRef, useState } from "react";
import type { FormEvent, ReactNode } from "react";
import { acceptOffer, bookRealClass, bookSeat, cancelBooking, checkCalendar, clearWorkspaceKey, connectCalendar, createCheckoutSession, createClass, createWorkspace, deleteWorkspace, exportWorkspace, joinWaitlist, leaveDemo, listBookings, listClasses, listOfferReceipts, loadDemo, loadOffer, loadOperationalMetrics, loadPublicClass, loadRuntimeStatus, loadWorkspace, publishClass, resetDemo, verifyBilling } from "./lib/api";
import type { BookingSummary, ClassSession, DemoData, OfferDeliveryStatus, OfferReceipt, OperationalMetrics, RealClass, Workspace } from "./lib/api";
import { finishSignIn, signIn, signedIn, signOut } from "./lib/auth";
import { routeForPath } from "./lib/routes";
import type { RouteInfo, WorkspaceSection } from "./lib/routes";

function useRoute() {
  const [route, setRoute] = useState(() => routeForPath(window.location.pathname));
  useEffect(() => {
    const change = () => setRoute(routeForPath(window.location.pathname));
    window.addEventListener("popstate", change);
    window.addEventListener("app:navigate", change);
    return () => {
      window.removeEventListener("popstate", change);
      window.removeEventListener("app:navigate", change);
    };
  }, []);
  return route;
}

function navigate(href: string) {
  window.history.pushState({}, "", href);
  window.dispatchEvent(new Event("app:navigate"));
  window.requestAnimationFrame(() => {
    const hash = new URL(href, window.location.origin).hash;
    if (hash) {
      document.querySelector(hash)?.scrollIntoView();
    } else {
      window.scrollTo({ top: 0, behavior: "instant" });
    }
  });
}

function AppLink({ href, className, children, onClick }: { href: string; className?: string; children: ReactNode; onClick?: () => void }) {
  return (
    <a href={href} className={className} onClick={(event) => {
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault();
      onClick?.();
      navigate(href);
    }}>
      {children}
    </a>
  );
}

function updateMetadata(route: RouteInfo) {
  document.title = route.title;
  document.querySelector<HTMLMetaElement>('meta[name="description"]')?.setAttribute("content", route.description);
  document.querySelector<HTMLLinkElement>('link[rel="canonical"]')?.setAttribute("href", `https://class-capacity-truth.sociobot.in${window.location.pathname}`);
  document.querySelector<HTMLMetaElement>('meta[property="og:title"]')?.setAttribute("content", route.title);
  document.querySelector<HTMLMetaElement>('meta[property="og:description"]')?.setAttribute("content", route.description);
  document.querySelector<HTMLMetaElement>('meta[name="twitter:title"]')?.setAttribute("content", route.title);
  document.querySelector<HTMLMetaElement>('meta[name="twitter:description"]')?.setAttribute("content", route.description);
}

function SiteHeader({ route }: { route: RouteInfo }) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuButton = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    setMenuOpen(false);
  }, [route.kind]);

  useEffect(() => {
    if (!menuOpen) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      setMenuOpen(false);
      menuButton.current?.focus();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [menuOpen]);

  const closeMenu = () => setMenuOpen(false);

  return (
    <header className="site-header">
      <AppLink className="wordmark" href="/" onClick={closeMenu}><RailMark /><span>Class Capacity Truth</span></AppLink>
      <button
        ref={menuButton}
        className="mobile-nav-toggle"
        type="button"
        aria-controls="main-navigation"
        aria-expanded={menuOpen}
        aria-label={`${menuOpen ? "Close" : "Open"} main menu`}
        onClick={() => setMenuOpen((current) => !current)}
      >
        <span aria-hidden="true">{menuOpen ? "Close menu" : "Open menu"}</span>
      </button>
      <nav id="main-navigation" className={`main-nav${menuOpen ? " is-open" : ""}`} aria-label="Main navigation">
        <AppLink href="/demo?demo=1" onClick={closeMenu}>Demo</AppLink>
        <a href={route.kind === "home" ? "#how-it-works" : "/#how-it-works"} onClick={closeMenu}>How it works</a>
        <AppLink href="/app" onClick={closeMenu}>School workspace</AppLink>
        <AppLink href="/privacy" onClick={closeMenu}>Privacy</AppLink>
      </nav>
    </header>
  );
}

export function App() {
  const route = useRoute();
  const firstRender = useRef(true);
  useEffect(() => {
    updateMetadata(route);
    if (firstRender.current) { firstRender.current = false; return; }
    const timer = window.setTimeout(() => document.querySelector<HTMLElement>("main h1")?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [route]);

  return (
    <>
      <a className="skip-link" href="#main" onClick={(event) => {
        event.preventDefault();
        const main = document.querySelector<HTMLElement>("main#main");
        window.history.replaceState(window.history.state, "", `${window.location.pathname}${window.location.search}#main`);
        main?.focus();
        main?.scrollIntoView({ block: "start" });
      }}>Skip to main content</a>
      <SiteHeader route={route} />
      <div className="route-announcer sr-only" aria-live="polite" aria-atomic="true">{route.title}</div>
      {route.kind === "home" && <HomePage />}
      {route.kind === "demo" && <DemoPage />}
      {route.kind === "booking" && (route.publicClassId?.startsWith("class_") ? <RealBookingPage publicClassId={route.publicClassId!} /> : <BookingPage publicClassId={route.publicClassId!} />)}
      {route.kind === "workspace" && <WorkspacePage section={route.workspaceSection ?? "classes"} classId={route.workspaceClassId} />}
      {route.kind === "authCallback" && <AuthCallbackPage />}
      {route.kind === "offer" && <OfferPage token={route.offerToken!} />}
      {route.kind === "privacy" && <PrivacyPage />}
      {route.kind === "terms" && <TermsPage />}
      {route.kind === "notFound" && <NotFoundPage />}
      <SiteFooter />
    </>
  );
}

function RailMark() {
  return (
    <svg className="wordmark-mark" aria-hidden="true" viewBox="0 0 54 32" width="54" height="32">
      <path d="M3 10h48M3 23h48" /><circle cx="15" cy="10" r="6" /><circle cx="39" cy="10" r="6" /><circle cx="26" cy="23" r="6" />
    </svg>
  );
}

function CheckoutButton({ label, className = "button secondary" }: { label: string; className?: string }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function checkout() {
    setBusy(true);
    setError("");
    try {
      window.location.assign(await createCheckoutSession());
    } catch (cause) {
      setError((cause as Error).message);
      setBusy(false);
    }
  }
  return <><button className={className} type="button" disabled={busy} onClick={() => void checkout()}>{busy ? "Opening checkout…" : label} <span className="sr-only">(external checkout)</span></button>{error && <p className="form-error" role="alert">{error}</p>}</>;
}

function HomePage() {
  return (
    <main id="main" tabIndex={-1}>
      <section className="hero page-width" aria-labelledby="home-title">
        <div className="hero-copy paper-panel">
          <p className="eyebrow">For small language schools</p>
          <h1 id="home-title" tabIndex={-1}>Show the right number of class seats</h1>
          <p className="lede">For small schools that need booking counts to match the class capacity they set.</p>
          <div className="action-row">
            <AppLink className="button primary" href="/demo?demo=1">Try it with sample data</AppLink>
            <span>Three sample classes open next.</span>
          </div>
          <ul className="plain-facts" aria-label="What the sample proves">
            <li>The demo stays separate and resets.</li>
            <li>No advertising trackers or analytics scripts.</li>
            <li>The school plan costs $99 each month.</li>
          </ul>
        </div>
        <div className="hero-object" aria-label="Eight seats: six confirmed and two open">
          <p><strong>2 seats open</strong><span>Level check: upper primary</span></p>
          <CapacityRail capacity={8} confirmed={6} label="Six confirmed seats and two open seats" />
          <div className="chalk-note" aria-hidden="true">8 seats − 6 booked = 2 open</div>
        </div>
      </section>

      <section className="product-preview section page-width" aria-labelledby="preview-title">
        <div><p className="eyebrow">Live seat preview</p><h2 id="preview-title">Count seats before taking a booking</h2><p>Book a sample seat and see the open count change.</p></div>
        <div className="preview-ledger">
          <PreviewRow name="Level check: upper primary" status="2 open" capacity={8} confirmed={6} />
          <PreviewRow name="Friday conversation group" status="Full" capacity={6} confirmed={6} />
          <PreviewRow name="Saturday assessment" status="Booking closed" capacity={10} confirmed={4} cutoff />
        </div>
      </section>

      <section id="how-it-works" className="section dark-section">
        <div className="page-width">
          <p className="eyebrow">How the sample works</p><h2>Follow one seat from open to booked</h2>
          <ol className="steps">
            <li><span>1</span><div><h3>Choose a class</h3><p>Compare an open class with full and closed examples.</p></div></li>
            <li><span>2</span><div><h3>Book one seat</h3><p>Enter a sample guardian name and example.org email.</p></div></li>
            <li><span>3</span><div><h3>See the count move</h3><p>The class changes from two open seats to one.</p></div></li>
          </ol>
        </div>
      </section>

      <section className="section two-column page-width" aria-labelledby="boundaries-title">
        <div>
          <p className="eyebrow">Class capacity only</p><h2 id="boundaries-title">Keep school records in your existing system</h2>
          <p>Use your existing school records for grades, attendance, tuition, and learning history.</p>
          <AppLink href="/privacy">Read how sample data is handled</AppLink>
        </div>
        <div id="school-plan" className="plan-note">
          <p className="eyebrow">School workspace</p><h2>Set a real class capacity</h2>
          <p>Create a persistent class, publish its booking link, compare calendar bookings, and record released-seat offers.</p>
          <AppLink className="button secondary" href="/app">Open school workspace</AppLink>
          <CheckoutButton className="text-action checkout-action" label="Open Sociobot checkout" />
        </div>
      </section>
    </main>
  );
}

function PreviewRow({ name, status, capacity, confirmed, cutoff = false }: { name: string; status: string; capacity: number; confirmed: number; cutoff?: boolean }) {
  return <div className={`preview-row${cutoff ? " is-cutoff" : ""}`}><div><strong>{name}</strong><span>{status}</span></div><CapacityRail capacity={capacity} confirmed={confirmed} compact label={`${name}: ${status}`} /></div>;
}

function DemoBanner({ onReset, resetting }: { onReset: () => void; resetting: boolean }) {
  async function startForReal() { await leaveDemo().catch(() => undefined); navigate("/app"); }
  return (
    <aside className="demo-banner" aria-label="Demo controls">
      <strong>Demo — sample data, nothing is saved</strong>
      <div><button className="button quiet" type="button" onClick={onReset} disabled={resetting}>{resetting ? "Resetting…" : "Reset demo"}</button><button className="button quiet" type="button" onClick={startForReal}>Start for real</button></div>
    </aside>
  );
}

function useDemoData() {
  const [data, setData] = useState<DemoData | null>(null);
  const [error, setError] = useState("");
  const [resetting, setResetting] = useState(false);
  const reload = () => { setError(""); loadDemo().then(setData).catch((cause: Error) => setError(cause.message)); };
  useEffect(reload, []);
  const reset = async () => {
    setResetting(true); setError("");
    try { setData(await resetDemo()); } catch (cause) { setError((cause as Error).message); } finally { setResetting(false); }
  };
  return { data, setData, error, resetting, reset, reload };
}

function DemoPage() {
  const demo = useDemoData();
  return (
    <><DemoBanner onReset={demo.reset} resetting={demo.resetting} />
      <main id="main" tabIndex={-1} className="page-width app-main">
        <p className="eyebrow">Bright Path Languages</p><h1 tabIndex={-1}>Check three sample classes</h1>
        <p className="lede">One class has seats. One is full. One has passed its booking cutoff.</p>
        <div className="demo-results" aria-busy={!demo.data && !demo.error}>
          {demo.error && <StatePanel tone="error" title="The sample did not load" detail={demo.error} action={<button className="button secondary" onClick={demo.reload}>Try loading again</button>} />}
          {!demo.data && !demo.error && <DemoLoadingState />}
          {demo.data && <div className="class-list" aria-label="Sample classes">{demo.data.classes.map((session) => <ClassCard key={session.publicId} session={session} />)}</div>}
        </div>
      </main>
    </>
  );
}

function DemoLoadingState() {
  const now = Math.floor(Date.now() / 1000);
  const placeholders: ClassSession[] = [
    { publicId: "loading-open", name: "Level check: upper primary", startsAt: now + 2 * 86_400, bookingCutoff: now + 86_400, timezone: "Europe/London", capacity: 8, confirmed: 6, openSeats: 2, availability: "available" },
    { publicId: "loading-full", name: "Friday conversation group", startsAt: now + 3 * 86_400, bookingCutoff: now + 2 * 86_400, timezone: "Europe/London", capacity: 6, confirmed: 6, openSeats: 0, availability: "full" },
    { publicId: "loading-cutoff", name: "Saturday assessment", startsAt: now + 7 * 86_400, bookingCutoff: now - 3_600, timezone: "Europe/London", capacity: 10, confirmed: 4, openSeats: 6, availability: "cutoff" }
  ];
  return <div className="class-list demo-loading-list" aria-live="polite"><p className="sr-only">Loading sample classes</p>{placeholders.map((session) => <ClassCard key={session.publicId} session={session} loading />)}</div>;
}

function ClassCard({ session, loading = false }: { session: ClassSession; loading?: boolean }) {
  return (
    <article className="class-card" aria-hidden={loading || undefined}>
      <div className="class-card-heading"><div><h2>{session.name}</h2><p>{formatStart(session.startsAt)} · {session.timezone}</p></div><strong className={`status status-${session.availability}`}>{availabilityText(session)}</strong></div>
      <CapacityRail capacity={session.capacity} confirmed={session.confirmed} label={`${session.confirmed} confirmed, ${session.openSeats} open`} />
      {session.availability === "available" && (loading ? <span className="button primary">Book this sample class</span> : <AppLink className="button primary" href={`/book/${session.publicId}`}>Book this sample class</AppLink>)}
      {session.availability === "full" && <><p className="state-explanation">This class is full. Choose the upper primary class to try a booking.</p>{loading ? <span className="text-action">View the full class</span> : <AppLink className="text-action" href={`/book/${session.publicId}`}>View the full class</AppLink>}</>}
      {session.availability === "cutoff" && <><p className="state-explanation">The booking cutoff has passed. Choose the upper primary class to try a booking.</p>{loading ? <span className="text-action">View the closed class</span> : <AppLink className="text-action" href={`/book/${session.publicId}`}>View the closed class</AppLink>}</>}
    </article>
  );
}

function BookingPage({ publicClassId }: { publicClassId: string }) {
  const demo = useDemoData();
  const [confirmed, setConfirmed] = useState(false);
  const session = demo.data?.classes.find((item) => item.publicId === publicClassId);
  function updateSession(next: ClassSession) {
    if (!demo.data) return;
    demo.setData({ ...demo.data, classes: demo.data.classes.map((item) => item.publicId === next.publicId ? next : item) });
    setConfirmed(true);
  }
  async function resetFromBooking() {
    setConfirmed(false);
    await demo.reset();
    navigate("/demo?demo=1");
  }
  return (
    <><DemoBanner onReset={() => { void resetFromBooking(); }} resetting={demo.resetting} />
      <main id="main" tabIndex={-1} className="page-width app-main booking-layout">
        <div><AppLink className="back-link" href="/demo?demo=1">← All sample classes</AppLink><p className="eyebrow">Bright Path Languages</p><h1 tabIndex={-1}>Book one sample seat</h1></div>
        {demo.error && <StatePanel tone="error" title="The class did not load" detail={demo.error} action={<button className="button secondary" onClick={demo.reload}>Try loading again</button>} />}
        {!demo.data && !demo.error && <LoadingState />}
        {demo.data && !session && <StatePanel tone="warning" title="This sample link has ended" detail="Reset the demo to get a fresh set of sample classes." action={<AppLink className="button secondary" href="/demo?demo=1">Open the demo</AppLink>} />}
        {session && (
          <section className="booking-card" aria-labelledby="class-name">
            <div className="booking-class-summary"><p className="eyebrow">Sample class</p><h2 id="class-name">{session.name}</h2><p>{formatStart(session.startsAt)} · {session.timezone}</p><strong className={`availability-large status-${session.availability}`}>{availabilityText(session)}</strong><CapacityRail capacity={session.capacity} confirmed={session.confirmed} label={`${session.confirmed} confirmed, ${session.openSeats} open`} animate={confirmed} /></div>
            {confirmed ? <StatePanel tone="success" title="Your sample seat is booked" detail={`${session.openSeats} ${session.openSeats === 1 ? "seat is" : "seats are"} now open in this class.`} action={<AppLink className="button secondary" href="/demo?demo=1">Check the other classes</AppLink>} />
              : session.availability === "available" ? <BookingForm session={session} onBooked={updateSession} />
              : session.availability === "full" ? <StatePanel tone="error" title="This class is full" detail="All sample seats are confirmed. Choose the upper primary class instead." action={<AppLink className="button secondary" href="/demo?demo=1">Choose another class</AppLink>} />
              : <StatePanel tone="warning" title="Booking has closed" detail="The sample cutoff has passed. Choose the upper primary class instead." action={<AppLink className="button secondary" href="/demo?demo=1">Choose another class</AppLink>} />}
          </section>
        )}
      </main>
    </>
  );
}

function BookingForm({ session, onBooked }: { session: ClassSession; onBooked: (next: ClassSession) => void }) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const key = useRef(crypto.randomUUID());
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault(); setPending(true); setError("");
    const form = new FormData(event.currentTarget);
    try { const result = await bookSeat(session.publicId, String(form.get("guardianName")), String(form.get("guardianEmail")), key.current); onBooked(result.class); }
    catch (cause) { setError((cause as Error).message); } finally { setPending(false); }
  }
  return (
    <form className="booking-form" onSubmit={submit}>
      <div><p className="eyebrow">Booking details</p><h2>Hold one seat</h2><p>Use sample details in this public demo.</p></div>
      <label>Guardian name<input name="guardianName" autoComplete="name" minLength={2} maxLength={80} required defaultValue="Alex Morgan" /></label>
      <label>Email address<input name="guardianEmail" type="email" autoComplete="email" maxLength={254} required defaultValue="alex.morgan@example.org" /></label>
      <label>Seats<input name="seats" value="1" readOnly aria-describedby="seat-help" /></label><p id="seat-help" className="field-help">The M1 sample books one seat at a time.</p>
      {error && <p className="form-error" role="alert">{error}</p>}
      <button className="button primary" type="submit" disabled={pending}>{pending ? "Booking sample seat…" : "Book one sample seat"}</button>
    </form>
  );
}

function WorkspacePage({ section, classId }: { section: WorkspaceSection; classId?: string }) {
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [classes, setClasses] = useState<RealClass[]>([]);
  const [bookings, setBookings] = useState<Record<string, BookingSummary[]>>({});
  const [offerReceipts, setOfferReceipts] = useState<OfferReceipt[]>([]);
  const [copiedReceipt, setCopiedReceipt] = useState("");
  const [isSignedIn, setIsSignedIn] = useState<boolean | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [emailDelivery, setEmailDelivery] = useState<"smtp" | "not_configured">("not_configured");
  const [operationalMetrics, setOperationalMetrics] = useState<OperationalMetrics | null>(null);
  const refresh = async () => { try { setWorkspace(await loadWorkspace()); const [nextClasses, nextReceipts] = await Promise.all([listClasses(), listOfferReceipts()]); setClasses(nextClasses); setOfferReceipts(nextReceipts); } catch { setWorkspace(null); setOfferReceipts([]); } };
  useEffect(() => { void loadRuntimeStatus().then((status) => setEmailDelivery(status.emailDelivery)); void signedIn().then((value) => { setIsSignedIn(value); if (value) void refresh(); }); }, []);
  useEffect(() => {
    if (section !== "operations" || !workspace) return;
    void loadOperationalMetrics().then(setOperationalMetrics).catch((cause: Error) => setError(cause.message));
  }, [section, workspace]);
  useEffect(() => {
    if (isSignedIn === null) return;
    const timer = window.setTimeout(() => document.querySelector<HTMLElement>("main h1")?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [isSignedIn, workspace, section]);
  async function start(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setBusy(true); setError(""); try { setWorkspace(await createWorkspace(String(new FormData(event.currentTarget).get("schoolName")))); await refresh(); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  async function addClass(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setBusy(true); setError(""); const formElement = event.currentTarget; const form = new FormData(formElement); const timezone = String(form.get("timezone")); const starts = zonedDateTimeToEpoch(String(form.get("startsAt")), timezone); const cutoff = zonedDateTimeToEpoch(String(form.get("cutoff")), timezone); try { await createClass({ name: String(form.get("name")), startsAt: starts, bookingCutoff: cutoff, timezone, capacity: Number(form.get("capacity")) }); formElement.reset(); await refresh(); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  async function calendar(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setBusy(true); setError(""); const form = new FormData(event.currentTarget); try { await connectCalendar(String(form.get("calendar")), String(form.get("feedUrl"))); const result = await checkCalendar(); setError(result.checked ? "Calendar connected and checked. Automatic checks run every five minutes." : "Calendar connected. Its first check is queued."); await refresh(); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  async function publish(id: string) { setBusy(true); try { await publishClass(id); await refresh(); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  async function showBookings(id: string) { try { setBookings((current) => ({ ...current, [id]: [] })); const rows = await listBookings(id); setBookings((current) => ({ ...current, [id]: rows })); } catch (cause) { setError((cause as Error).message); } }
  async function release(id: string, booking: BookingSummary) { if (!window.confirm(`Cancel ${booking.guardianName}'s confirmed seat?`)) return; setBusy(true); try { const result = await cancelBooking(id, booking.id); setError(result.offerToken ? (result.deliveryStatus === "email_queued" ? "The selected booking was cancelled. The released-seat email is queued." : "The selected booking was cancelled. Copy the one-click offer below and send it through the school’s usual email or messaging service.") : "The selected booking was cancelled. No guardian is waiting."); await refresh(); await showBookings(id); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  async function copyOffer(receipt: OfferReceipt) { try { await navigator.clipboard.writeText(receipt.offerUrl); setCopiedReceipt(receipt.id); } catch { setError("The offer could not be copied. Select the offer URL and copy it manually."); } }
  async function downloadData() { const data = await exportWorkspace(); const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: "application/json" })); const anchor = document.createElement("a"); anchor.href = url; anchor.download = "class-capacity-truth-export.json"; anchor.click(); URL.revokeObjectURL(url); }
  async function removeData() { if (!window.confirm(`Delete ${workspace?.schoolName ?? "this workspace"} and all its data?`)) return; await deleteWorkspace(); clearWorkspaceKey(); setWorkspace(null); setClasses([]); }
  async function billing(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setBusy(true); setError(""); try { setWorkspace(await verifyBilling(String(new FormData(event.currentTarget).get("license")))); setError("The school plan is active."); } catch (cause) { setError((cause as Error).message); } finally { setBusy(false); } }
  if (isSignedIn === null) return <main id="main" tabIndex={-1} className="page-width app-main"><h1 tabIndex={-1}>{workspaceHeading(section, "loading")}</h1><LoadingState /></main>;
  if (!isSignedIn) return <main id="main" tabIndex={-1} className="page-width app-main"><p className="eyebrow">School workspace</p><h1 tabIndex={-1}>{section === "classes" ? "Sign in to manage class capacity" : workspaceHeading(section, "signedOut")}</h1><p className="lede">Owners, operators, and viewers use Sociobot’s Microsoft sign-in. Guardian booking pages stay public.</p><button className="button primary" onClick={() => void signIn()}>Sign in with Sociobot</button><section className="plan-note"><h2>School plan</h2><p><strong>$99 per school each month.</strong> It includes calendar checks and one-click released-seat offers.</p><p>{emailDelivery === "smtp" ? "This deployment can send offer email." : "This deployment creates a copyable offer for staff to send through the school’s usual email or messaging service."}</p><CheckoutButton label="Start the $99 monthly plan" /></section></main>;
  if (!workspace) return <main id="main" tabIndex={-1} className="page-width app-main"><p className="eyebrow">School workspace</p><h1 tabIndex={-1}>{section === "classes" ? "Create your school workspace" : workspaceHeading(section, "empty")}</h1><p className="lede">Set a class capacity and cutoff, then publish a guardian booking link.</p><form className="booking-form workspace-form" onSubmit={start}><label>School name<input name="schoolName" required minLength={2} maxLength={100} placeholder="Bright Path Languages" /></label>{error && <p className="form-error" role="alert">{error}</p>}<button className="button primary" disabled={busy}>{busy ? "Creating workspace…" : "Create school workspace"}</button><p className="field-help">Your Microsoft sign-in identifies this workspace. The browser key only selects it.</p></form><button className="button quiet" onClick={() => void signOut()}>Sign out</button></main>;
  if (section !== "classes") return <WorkspaceSectionPage section={section} classId={classId} workspace={workspace} classes={classes} bookings={bookings} offerReceipts={offerReceipts} copiedReceipt={copiedReceipt} operationalMetrics={operationalMetrics} error={error} busy={busy} emailDelivery={emailDelivery} onCalendar={calendar} onPublish={publish} onShowBookings={showBookings} onRelease={release} onCopy={copyOffer} onBilling={billing} onDownload={downloadData} onDelete={removeData} />;
  return <main id="main" tabIndex={-1} className="page-width app-main"><p className="eyebrow">{workspace.schoolName}</p><h1 tabIndex={-1}>Manage class capacity</h1><WorkspaceNavigation active="classes" /><p className="lede">Publish only a count you can reconcile. A mismatch is shown as attention, never changed automatically.</p>{emailDelivery === "not_configured" && <p className="state-explanation" role="status">This deployment does not send email. Cancelling creates a one-click offer below. Copy it and send it through the school’s usual email or messaging service.</p>}{error && <p className="form-error" role="alert">{error}</p>}
    <section className="workspace-grid" aria-label="Set up a class and calendar"><form className="booking-form" onSubmit={addClass}><h2>Create a class</h2><label>Class name<input name="name" required minLength={2} maxLength={120} /></label><label>School time zone<select name="timezone" defaultValue="Europe/London"><option value="Europe/London">Europe/London</option><option value="America/New_York">America/New_York</option><option value="America/Los_Angeles">America/Los_Angeles</option><option value="Asia/Kolkata">Asia/Kolkata</option><option value="Australia/Sydney">Australia/Sydney</option></select></label><label>Starts at<input name="startsAt" type="datetime-local" required /></label><label>Booking cutoff<input name="cutoff" type="datetime-local" required /></label><label>Capacity<input name="capacity" type="number" min="1" max="500" required defaultValue="8" /></label><button className="button primary" disabled={busy}>Create class</button></form><form className="booking-form" onSubmit={calendar}><h2>Connect one calendar</h2><p>Paste a private HTTPS iCalendar feed. It is encrypted and checked every five minutes.</p><label>Calendar label<input name="calendar" required placeholder="School bookings calendar" /></label><label>iCalendar feed URL<input name="feedUrl" type="url" inputMode="url" required placeholder="https://calendar.google.com/calendar/ical/…" /></label><button className="button secondary" disabled={busy}>Connect and check calendar</button></form></section>
    <section className="class-list workspace-list" aria-label="Real classes"><h2>Published classes and checks</h2>{classes.length === 0 && <p>No classes yet. Create one above.</p>}{classes.map((item) => <article className="class-card" key={item.id}><div className="class-card-heading"><div><h3>{item.name}</h3><p>{formatStart(item.startsAt, item.timezone)} · {item.timezone}</p></div><strong className={`status status-${item.availability}`}>{availabilityText(item)}</strong></div><CapacityRail capacity={item.capacity} confirmed={item.confirmed} label={`${item.confirmed} confirmed, ${item.openSeats} open`} /><p><AppLink className="text-action" href={`/app/classes/${item.id}`}>Open class details</AppLink></p>{item.published ? <><p><strong>Guardian link:</strong> <AppLink href={`/book/${item.publicId}`}>Open booking page</AppLink></p><p>{item.reconciliationStatus === "attention" ? `Attention: calendar says ${item.calendarConfirmed}, local ledger says ${item.confirmed}.` : item.reconciliationStatus === "matched" ? "Calendar check matches the local seat ledger." : "Waiting for the first automatic calendar check."}</p><button className="button secondary" type="button" onClick={() => void showBookings(item.id)}>Choose a booking to cancel</button>{bookings[item.id] && <ul className="booking-list">{bookings[item.id].length === 0 ? <li>No confirmed bookings remain.</li> : bookings[item.id].map((booking) => <li key={booking.id}><span><strong>{booking.guardianName}</strong><small>{booking.guardianEmail}</small></span><button className="button quiet" disabled={busy} onClick={() => void release(item.id, booking)}>Cancel {booking.guardianName} booking</button></li>)}</ul>}</> : <button className="button primary" type="button" disabled={busy} onClick={() => void publish(item.id)}>Publish guardian link</button>}</article>)}</section>
    <OfferReceipts receipts={offerReceipts} copiedReceipt={copiedReceipt} onCopy={copyOffer} />
    <section className="workspace-grid settings-grid"><form className="booking-form" onSubmit={billing}><h2>School plan</h2><p><strong>$99 per school each month.</strong> Status: {workspace.subscriptionStatus}.</p><CheckoutButton label="Open Sociobot checkout" /><label>Purchase token<input name="license" required autoComplete="off" /></label><button className="button secondary" disabled={busy}>Verify purchase token</button></form><div className="booking-form"><h2>Your school data</h2><p>Owners can export all class and guardian data or delete the whole workspace.</p><button className="button secondary" onClick={() => void downloadData()}>Export school data</button><button className="button danger" onClick={() => void removeData()}>Delete school workspace</button><button className="button quiet" onClick={() => void signOut()}>Sign out</button></div></section></main>;
}

function workspaceHeading(section: WorkspaceSection, state: "loading" | "signedOut" | "empty") {
  const base: Record<WorkspaceSection, string> = {
    classes: "Manage class capacity",
    classDetail: "Check this class capacity",
    reconciliation: "Check calendar differences",
    waitlist: "Manage released-seat offers",
    settings: "Manage school settings",
    billing: "Manage school billing",
    data: "Export or delete school data",
    operations: "Review capacity operations"
  };
  if (state === "loading") return base[section];
  if (state === "signedOut") return section === "classes" ? "Sign in to manage class capacity" : `Sign in to ${base[section].toLowerCase()}`;
  return section === "classes" ? "Create your school workspace" : `${base[section]} after you create a school workspace`;
}

function WorkspaceNavigation({ active }: { active: WorkspaceSection }) {
  const activeSection = active === "classDetail" ? "classes" : active;
  const links: Array<{ href: string; label: string; section: WorkspaceSection }> = [
    { href: "/app", label: "Classes", section: "classes" },
    { href: "/app/reconciliation", label: "Calendar checks", section: "reconciliation" },
    { href: "/app/waitlist", label: "Waitlist offers", section: "waitlist" },
    { href: "/app/operations", label: "Operations", section: "operations" },
    { href: "/app/settings", label: "Settings", section: "settings" }
  ];
  return <nav className="workspace-nav" aria-label="School workspace sections">{links.map((link) => <AppLink key={link.href} href={link.href} className={activeSection === link.section ? "is-current" : ""}>{link.label}</AppLink>)}</nav>;
}

function WorkspaceSectionPage({ section, classId, workspace, classes, bookings, offerReceipts, copiedReceipt, operationalMetrics, error, busy, emailDelivery, onCalendar, onPublish, onShowBookings, onRelease, onCopy, onBilling, onDownload, onDelete }: {
  section: WorkspaceSection;
  classId?: string;
  workspace: Workspace;
  classes: RealClass[];
  bookings: Record<string, BookingSummary[]>;
  offerReceipts: OfferReceipt[];
  copiedReceipt: string;
  operationalMetrics: OperationalMetrics | null;
  error: string;
  busy: boolean;
  emailDelivery: "smtp" | "not_configured";
  onCalendar: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onPublish: (id: string) => Promise<void>;
  onShowBookings: (id: string) => Promise<void>;
  onRelease: (id: string, booking: BookingSummary) => Promise<void>;
  onCopy: (receipt: OfferReceipt) => Promise<void>;
  onBilling: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  onDownload: () => Promise<void>;
  onDelete: () => Promise<void>;
}) {
  const detail = section === "classDetail" ? classes.find((item) => item.id === classId) : undefined;
  const headings: Record<WorkspaceSection, { heading: string; lede: string }> = {
    classes: { heading: "Manage class capacity", lede: "Publish only a count you can reconcile." },
    classDetail: { heading: "Check this class capacity", lede: "Review bookings and the count for this class." },
    reconciliation: { heading: "Check calendar differences", lede: "A difference is shown for review and never changes confirmed seats automatically." },
    waitlist: { heading: "Manage released-seat offers", lede: "Review durable offer receipts before contacting a waiting guardian." },
    settings: { heading: "Manage school settings", lede: "Open billing or data controls for this school." },
    billing: { heading: "Manage school billing", lede: "Open Sociobot checkout to start or manage the school plan." },
    data: { heading: "Export or delete school data", lede: "Owners can download school data or remove this workspace." },
    operations: { heading: "Review capacity operations", lede: "See aggregate service health and the capacity signals that need staff attention." }
  };
  const copy = headings[section];
  const classDetails = detail ? <article className="class-card"><div className="class-card-heading"><div><h2>{detail.name}</h2><p>{formatStart(detail.startsAt, detail.timezone)} · {detail.timezone}</p></div><strong className={`status status-${detail.availability}`}>{availabilityText(detail)}</strong></div><CapacityRail capacity={detail.capacity} confirmed={detail.confirmed} label={`${detail.confirmed} confirmed, ${detail.openSeats} open`} />{detail.published ? <p><strong>Guardian link:</strong> <AppLink href={`/book/${detail.publicId}`}>Open booking page</AppLink></p> : <button className="button primary" type="button" disabled={busy} onClick={() => void onPublish(detail.id)}>Publish guardian link</button>}<p>{detail.reconciliationStatus === "attention" ? `Attention: calendar says ${detail.calendarConfirmed}, local ledger says ${detail.confirmed}.` : detail.reconciliationStatus === "matched" ? "Calendar check matches the local seat ledger." : "Waiting for the first automatic calendar check."}</p>{detail.published && <><button className="button secondary" type="button" onClick={() => void onShowBookings(detail.id)}>Choose a booking to cancel</button>{bookings[detail.id] && <ul className="booking-list">{bookings[detail.id].length === 0 ? <li>No confirmed bookings remain.</li> : bookings[detail.id].map((booking) => <li key={booking.id}><span><strong>{booking.guardianName}</strong><small>{booking.guardianEmail}</small></span><button className="button quiet" disabled={busy} onClick={() => void onRelease(detail.id, booking)}>Cancel {booking.guardianName} booking</button></li>)}</ul>}</>}</article> : <StatePanel tone="warning" title="This class is not in the current workspace" detail="Return to classes and choose a class that belongs to this school." action={<AppLink className="button secondary" href="/app">Open classes</AppLink>} />;
  return <main id="main" tabIndex={-1} className="page-width app-main"><p className="eyebrow">{workspace.schoolName}</p><h1 tabIndex={-1}>{copy.heading}</h1><WorkspaceNavigation active={section} /><p className="lede">{copy.lede}</p>{emailDelivery === "not_configured" && section === "waitlist" && <p className="state-explanation" role="status">This deployment does not send email. Copy the offer and send it through the school’s usual email or messaging service.</p>}{error && <p className="form-error" role="alert">{error}</p>}
    {section === "classDetail" && classDetails}
    {section === "reconciliation" && <><section className="workspace-grid" aria-label="Calendar checks"><form className="booking-form" onSubmit={onCalendar}><h2>Connect one calendar</h2><p>Paste a private HTTPS iCalendar feed. It is encrypted and checked every five minutes.</p><label>Calendar label<input name="calendar" required placeholder="School bookings calendar" /></label><label>iCalendar feed URL<input name="feedUrl" type="url" inputMode="url" required placeholder="https://calendar.google.com/calendar/ical/…" /></label><button className="button primary" disabled={busy}>Connect and check calendar</button></form><section className="booking-form" aria-labelledby="calendar-status-title"><h2 id="calendar-status-title">Current class checks</h2>{classes.length === 0 ? <p>Create a class before connecting a calendar.</p> : <ul className="check-list">{classes.map((item) => <li key={item.id}><AppLink href={`/app/classes/${item.id}`}>{item.name}</AppLink><span>{item.reconciliationStatus === "attention" ? `Attention: calendar says ${item.calendarConfirmed}, local ledger says ${item.confirmed}.` : item.reconciliationStatus === "matched" ? "Matched" : "Waiting for first check"}</span></li>)}</ul>}</section></section></>}
    {section === "waitlist" && <OfferReceipts receipts={offerReceipts} copiedReceipt={copiedReceipt} onCopy={onCopy} />}
    {section === "settings" && <section className="workspace-grid settings-grid"><section className="booking-form"><h2>Billing</h2><p>Verify a subscription or open the hosted Sociobot checkout.</p><AppLink className="button secondary" href="/app/settings/billing">Open billing</AppLink></section><section className="booking-form"><h2>School data</h2><p>Export or delete the workspace from the owner-only data controls.</p><AppLink className="button secondary" href="/app/settings/data">Open data controls</AppLink></section></section>}
    {section === "billing" && <section className="booking-form"><h2>School plan</h2><p><strong>$99 per school each month.</strong> Status: {workspace.subscriptionStatus}.</p><CheckoutButton label="Open Sociobot checkout" /><form onSubmit={onBilling}><label>Purchase token<input name="license" required autoComplete="off" /></label><button className="button secondary" disabled={busy}>Verify purchase token</button></form></section>}
    {section === "data" && <section className="booking-form"><h2>Your school data</h2><p>Owners can export all class and guardian data or delete the whole workspace.</p><button className="button secondary" onClick={() => void onDownload()}>Export school data</button><button className="button danger" onClick={() => void onDelete()}>Delete school workspace</button></section>}
    {section === "operations" && <OperationalMetricsPanel metrics={operationalMetrics} />}
  </main>;
}

function OperationalMetricsPanel({ metrics }: { metrics: OperationalMetrics | null }) {
  if (!metrics) return <LoadingState />;
  const averageLatency = metrics.requests === 0 ? 0 : Math.round(metrics.totalLatencyMilliseconds / metrics.requests);
  return <section className="operations-panel" aria-labelledby="operational-metrics-title"><h2 id="operational-metrics-title">Operational metrics</h2><p>These aggregate counts contain no guardian names, email addresses, class names, or booking links.</p><dl className="metric-grid"><div><dt>Requests</dt><dd>{metrics.requests}</dd></div><div><dt>Server errors</dt><dd>{metrics.serverErrors}</dd></div><div><dt>Average response</dt><dd>{averageLatency} ms</dd></div><div><dt>Slowest response</dt><dd>{metrics.maxLatencyMilliseconds} ms</dd></div><div><dt>Calendar check lag</dt><dd>{metrics.calendarJobLagSeconds} s</dd></div><div><dt>Unresolved differences</dt><dd>{metrics.unresolvedDiscrepancies}</dd></div><div><dt>Offers accepted</dt><dd>{metrics.offersAccepted} of {metrics.offersCreated}</dd></div><div><dt>Offer conversion</dt><dd>{Math.round(metrics.offerConversionRatio * 100)}%</dd></div></dl><h3>Escalation thresholds</h3><ul><li>Investigate any server error and any unresolved public capacity difference.</li><li>Check a calendar connection when lag exceeds 10 minutes.</li><li>Review monthly availability against the 99.9% API target.</li></ul></section>;
}

function deliveryText(status: OfferDeliveryStatus) {
  const labels: Record<OfferDeliveryStatus, string> = {
    ready_to_copy: "Ready to share — no email was sent.",
    email_queued: "Email queued.",
    email_sent: "Email sent.",
    email_failed: "Email failed — copy and send this offer.",
    accepted: "Seat accepted.",
    expired: "Offer expired.",
    legacy_recorded: "Recorded before delivery receipts were available."
  };
  return labels[status];
}

function OfferReceipts({ receipts, copiedReceipt, onCopy }: { receipts: OfferReceipt[]; copiedReceipt: string; onCopy: (receipt: OfferReceipt) => void }) {
  return <section className="offer-receipts" aria-labelledby="offer-receipts-title"><h2 id="offer-receipts-title">Released-seat offer receipts</h2>{receipts.length === 0 ? <p>No released-seat offers yet. New offers and their delivery state appear here.</p> : <div className="receipt-list">{receipts.map((receipt) => <article className="offer-receipt" key={receipt.id}><h3>{receipt.className}</h3><p><strong>{receipt.recipientName}</strong></p><p className="receipt-state">{deliveryText(receipt.deliveryStatus)}</p><p>Expires {formatStart(receipt.expiresAt)}.</p><label htmlFor={`offer-url-${receipt.id}`}>One-click offer URL</label><input id={`offer-url-${receipt.id}`} value={receipt.offerUrl} readOnly onFocus={(event) => event.currentTarget.select()} /><button className="button secondary" type="button" onClick={() => onCopy(receipt)}>Copy offer</button><span className="copy-result" role="status" aria-live="polite">{copiedReceipt === receipt.id ? "Offer copied." : ""}</span></article>)}</div>}</section>;
}

function RealBookingPage({ publicClassId }: { publicClassId: string }) {
  const [session, setSession] = useState<RealClass | null>(null); const [error, setError] = useState(""); const [complete, setComplete] = useState(""); const [pending, setPending] = useState(false); const key = useRef(crypto.randomUUID());
  useEffect(() => { loadPublicClass(publicClassId).then(setSession).catch((cause: Error) => setError(cause.message)); }, [publicClassId]);
  async function submit(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setPending(true); setError(""); const form = new FormData(event.currentTarget); try { setSession(await bookRealClass(publicClassId, String(form.get("guardianName")), String(form.get("guardianEmail")), key.current)); setComplete("Your seat is confirmed."); } catch (cause) { setError((cause as Error).message); } finally { setPending(false); } }
  async function waitlist(event: FormEvent<HTMLFormElement>) { event.preventDefault(); setPending(true); setError(""); const form = new FormData(event.currentTarget); try { await joinWaitlist(publicClassId, String(form.get("guardianName")), String(form.get("guardianEmail")), key.current); setComplete("You are on the waitlist. The school can send you one 24-hour offer if a seat is released."); } catch (cause) { setError((cause as Error).message); } finally { setPending(false); } }
  return <main id="main" tabIndex={-1} className="page-width app-main booking-layout"><div><p className="eyebrow">Guardian booking</p><h1 tabIndex={-1}>Book a class seat</h1></div>{error && !session && <StatePanel tone="error" title="This booking link is unavailable" detail={error} />} {!session && !error && <LoadingState />}{session && <section className="booking-card"><div className="booking-class-summary"><p className="eyebrow">{session.name}</p><h2>{availabilityText(session)}</h2><p>{formatStart(session.startsAt, session.timezone)} · {session.timezone}</p><CapacityRail capacity={session.capacity} confirmed={session.confirmed} label={`${session.confirmed} confirmed, ${session.openSeats} open`} /></div>{complete ? <StatePanel tone="success" title="Booking update" detail={complete} /> : session.availability === "available" ? <form className="booking-form" onSubmit={submit}><h2>Your details</h2><label>Guardian name<input name="guardianName" required minLength={2} maxLength={80} autoComplete="name" /></label><label>Email address<input name="guardianEmail" type="email" required autoComplete="email" /></label>{error && <p className="form-error" role="alert">{error}</p>}<button className="button primary" disabled={pending}>{pending ? "Booking…" : "Book this seat"}</button></form> : <form className="booking-form" onSubmit={waitlist}><h2>Join the waitlist</h2><p>This class cannot take a confirmed booking now.</p><label>Guardian name<input name="guardianName" required minLength={2} maxLength={80} /></label><label>Email address<input name="guardianEmail" type="email" required /></label><p className="field-help">By joining, you agree to receive one released-seat offer from the school.</p>{error && <p className="form-error" role="alert">{error}</p>}<button className="button primary" disabled={pending}>{pending ? "Joining waitlist…" : "Join waitlist"}</button></form>}</section>}</main>;
}

function OfferPage({ token }: { token: string }) {
  const [offer, setOffer] = useState<{ class: RealClass; expiresAt: number } | null>(null); const [message, setMessage] = useState("");
  useEffect(() => { loadOffer(token).then(setOffer).catch((cause: Error) => setMessage(cause.message)); }, [token]);
  async function accept() { try { const result = await acceptOffer(token); setMessage(`Your released seat is confirmed. ${result.openSeats} seats remain open.`); } catch (cause) { setMessage((cause as Error).message); } }
  return <main id="main" tabIndex={-1} className="page-width app-main"><p className="eyebrow">Released-seat offer</p><h1 tabIndex={-1}>Claim your available class seat</h1>{!offer && !message && <LoadingState />}{offer && !message && <section className="state-panel success"><h2>{offer.class.name}</h2><p>This offer expires {formatStart(offer.expiresAt)}.</p><button className="button primary" onClick={() => void accept()}>Accept this seat</button></section>}{message && <StatePanel tone="success" title="Offer update" detail={message} />}</main>;
}

function AuthCallbackPage() {
  const [error, setError] = useState("");
  useEffect(() => { void finishSignIn().then((ok) => { if (ok) navigate("/app"); else setError("Sign-in did not finish. Return to the workspace and try again."); }).catch((cause: Error) => setError(cause.message)); }, []);
  return <main id="main" tabIndex={-1} className="page-width app-main"><h1 tabIndex={-1}>Finish staff sign in</h1>{error ? <StatePanel tone="error" title="Sign-in stopped" detail={error} action={<AppLink className="button secondary" href="/app">Return to the workspace</AppLink>} /> : <LoadingState />}</main>;
}

function CapacityRail({ capacity, confirmed, label, compact = false, animate = false }: { capacity: number; confirmed: number; label: string; compact?: boolean; animate?: boolean }) {
  return (
    <div className={`capacity-rail${compact ? " compact" : ""}${animate ? " just-booked" : ""}`} role="img" aria-label={label} tabIndex={0}>
      <span className="rail-line" aria-hidden="true" />
      {Array.from({ length: capacity }, (_, index) => <span className={`seat-bead ${index < confirmed ? "confirmed" : "open"}`} aria-hidden="true" key={index}>{index + 1}</span>)}
    </div>
  );
}

function StatePanel({ tone, title, detail, action }: { tone: "success" | "warning" | "error"; title: string; detail: string; action?: ReactNode }) {
  return <section className={`state-panel ${tone}`} aria-live={tone === "error" ? "assertive" : "polite"}><h2>{title}</h2><p>{detail}</p>{action}</section>;
}

function LoadingState() { return <section className="loading-state" aria-live="polite"><span aria-hidden="true" /><div><h2>Loading sample classes</h2><p>The seat rails will appear here.</p></div></section>; }

function PrivacyPage() {
  return (
    <main id="main" tabIndex={-1} className="page-width legal-page"><p className="eyebrow">Last updated 28 August 2026</p><h1 tabIndex={-1}>Privacy for bookings and the demo</h1><p className="lede">The public demo is temporary. School bookings use encrypted contact fields and staff access controls.</p>
      <h2>What the demo stores</h2><p>It stores the chosen sample class and a random browser identifier. The name and email you enter are checked, then discarded.</p>
      <h2>How long it stays</h2><p>A demo expires after 24 hours. Reset demo removes its bookings and starts the sample again. Start for real removes that browser’s demo.</p>
      <h2>School booking data</h2><p>The school is the data controller. Sociobot processes guardian names and email addresses to manage seats and create requested offers.</p><p>Contact fields are encrypted at rest. They are erased 90 days after collection. Audit records may remain without the contact fields.</p>
      <h2>Who receives it</h2><p>This service handles bookings. A configured email relay receives offer contact data only when the school enables email delivery.</p><p>There are no advertising trackers or analytics scripts.</p>
      <h2>Your choices</h2><p>Workspace owners can export or delete school data. Guardians may ask their school or <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a> for access or deletion.</p>
      <h2>Regional rights</h2><p>Depending on your region, you may ask to access, correct, export, restrict, object to, or delete personal data.</p>
    </main>
  );
}

function TermsPage() {
  return (
    <main id="main" tabIndex={-1} className="page-width legal-page"><p className="eyebrow">Last updated 28 August 2026</p><h1 tabIndex={-1}>Terms for Class Capacity Truth</h1><p className="lede">These terms cover the public demo and school workspace.</p>
      <h2>Use the demo for evaluation</h2><p>Use sample details only in the demo. The school workspace is for a school’s own class configuration and booking links.</p>
      <h2>Availability</h2><p>The sample may change or be removed. It is provided without a service commitment.</p>
      <h2>School workspace</h2><p>Schools set capacity, cutoff, booking links, calendar checks, and waitlist offers. Staff access uses Sociobot Microsoft Entra sign-in.</p>
      <h2>School plan</h2><p>The school plan costs $99 each month. Sociobot is the merchant of record and handles checkout, cancellation, and refunds.</p>
      <h2>Contact</h2><p>Questions may be sent to <a href="mailto:support@sociobot.in">support@sociobot.in</a>.</p>
    </main>
  );
}

function NotFoundPage() {
  return <main id="main" tabIndex={-1} className="page-width not-found"><div className="lost-bead" aria-hidden="true"><span /><i /></div><p className="eyebrow">404 · bead off the rail</p><h1 tabIndex={-1}>This page has no class</h1><p>The address may be old, or the sample link may have ended.</p><AppLink className="button primary" href="/">Return to the seat count</AppLink></main>;
}

function SiteFooter() {
  return <footer className="site-footer"><div><strong>Class Capacity Truth</strong><span>Seat counts for small schools.</span></div><nav aria-label="Footer navigation"><AppLink href="/privacy">Privacy</AppLink><AppLink href="/terms">Terms</AppLink><a href="https://sociobot.in">Built by Param Factory <span className="sr-only">(external site)</span></a></nav><p>Version 0.1.0 · Abacus visual system.</p></footer>;
}

function availabilityText(session: ClassSession) {
  if (session.availability === "full") return "Full · 0 seats open";
  if (session.availability === "cutoff") return `Booking closed · ${session.openSeats} unbooked`;
  return `${session.openSeats} ${session.openSeats === 1 ? "seat" : "seats"} open`;
}

function formatStart(timestamp: number, timezone = "Europe/London") {
  return new Intl.DateTimeFormat("en-GB", { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", timeZone: timezone }).format(new Date(timestamp * 1000));
}

export function zonedDateTimeToEpoch(value: string, timezone: string) {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/);
  if (!match) throw new Error("Choose a complete local date and time.");
  const desired = Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3]), Number(match[4]), Number(match[5]));
  let guess = desired;
  const formatter = new Intl.DateTimeFormat("en-CA", { timeZone: timezone, year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hourCycle: "h23" });
  for (let attempt = 0; attempt < 2; attempt += 1) {
    const parts = Object.fromEntries(formatter.formatToParts(new Date(guess)).map((part) => [part.type, part.value]));
    const rendered = Date.UTC(Number(parts.year), Number(parts.month) - 1, Number(parts.day), Number(parts.hour), Number(parts.minute));
    guess += desired - rendered;
  }
  return Math.floor(guess / 1000);
}
