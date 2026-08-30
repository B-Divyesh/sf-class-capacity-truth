import fs from "node:fs/promises";
import path from "node:path";
import AxeBuilder from "@axe-core/playwright";
import { chromium } from "@playwright/test";

const baseURL = process.env.BASE_URL ?? "https://class-capacity-truth.sociobot.in";
const evidenceDir = process.env.EVIDENCE_DIR ?? ".factory/evidence-repair-16/live";
const expectedOrigin = new URL(baseURL).origin;
const failures = [];
const report = { baseURL, consoleErrors: [], pageErrors: [], axe: {}, privacyOrigins: [], desktop: {}, mobile: {}, identity: {}, offline: {} };
const check = (condition, message) => { if (!condition) failures.push(message); };

await fs.mkdir(evidenceDir, { recursive: true });
const browser = await chromium.launch();
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on("console", (message) => { if (message.type() === "error") report.consoleErrors.push(message.text()); });
  page.on("pageerror", (error) => report.pageErrors.push(String(error)));
  await page.goto(baseURL, { waitUntil: "networkidle" });
  report.desktop = await page.evaluate(() => ({
    title: document.title,
    lang: document.documentElement.lang,
    h1: document.querySelectorAll("h1").length,
    main: document.querySelectorAll("main").length
  }));
  check(report.desktop.title === "Class Capacity Truth — Show the right seat count", "unexpected home title");
  check(report.desktop.lang === "en" && report.desktop.h1 === 1 && report.desktop.main === 1, "home semantics failed");
  await page.keyboard.press("Tab");
  check(await page.getByRole("link", { name: "Skip to main content" }).evaluate((node) => node === document.activeElement), "skip link did not receive focus");
  await page.keyboard.press("Enter");
  check(await page.locator("main").evaluate((node) => node === document.activeElement), "skip link did not focus main");
  await page.screenshot({ path: path.join(evidenceDir, "home-desktop.png"), fullPage: true });
  const homeAxe = await new AxeBuilder({ page }).analyze();
  report.axe.home = homeAxe.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? "")).length;

  await page.getByRole("link", { name: "Try it with sample data" }).click();
  await page.getByRole("article").first().waitFor();
  check((await page.getByRole("article").count()) === 3, "demo did not load three sample classes");
  check(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo isolation banner missing");
  const demoAxe = await new AxeBuilder({ page }).analyze();
  report.axe.demo = demoAxe.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? "")).length;
  await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  const bookingResult = page.getByText("1 seat is now open in this class.");
  await bookingResult.waitFor({ state: "visible" });
  check(await bookingResult.isVisible(), "live sample booking did not update seats");
  await page.screenshot({ path: path.join(evidenceDir, "booking-success-desktop.png"), fullPage: true });
  await context.close();

  const mobileContext = await browser.newContext({ viewport: { width: 390, height: 844 }, colorScheme: "dark", reducedMotion: "reduce" });
  const mobile = await mobileContext.newPage();
  await mobile.goto(`${baseURL}/demo?demo=1`, { waitUntil: "networkidle" });
  const menu = mobile.getByRole("button", { name: "Open main menu" });
  const box = await menu.boundingBox();
  await menu.focus();
  await mobile.keyboard.press("Enter");
  check(await mobile.getByRole("navigation", { name: "Main navigation" }).isVisible(), "mobile keyboard menu did not open");
  await mobile.keyboard.press("Escape");
  report.mobile = await mobile.evaluate(() => ({
    noOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth,
    animationDurations: [...new Set([...document.querySelectorAll("*")].flatMap((node) => getComputedStyle(node).animationDuration.split(",")))],
    transitionDurations: [...new Set([...document.querySelectorAll("*")].flatMap((node) => getComputedStyle(node).transitionDuration.split(",")))]
  }));
  report.mobile.menu = box;
  check(Boolean(box && box.width >= 44 && box.height >= 44), "mobile menu target is below 44px");
  check(report.mobile.noOverflow, "390px page has horizontal overflow");
  check(report.mobile.animationDurations.every((value) => value.trim() === "0s"), "reduced motion left an animation enabled");
  check(report.mobile.transitionDurations.every((value) => value.trim() === "0s"), "reduced motion left a transition enabled");
  const mobileAxe = await new AxeBuilder({ page: mobile }).analyze();
  report.axe.mobileDarkReduced = mobileAxe.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? "")).length;
  await mobile.screenshot({ path: path.join(evidenceDir, "demo-mobile-dark-reduced.png"), fullPage: true });
  await mobileContext.close();

  const privacyContext = await browser.newContext();
  const privacyPage = await privacyContext.newPage();
  const origins = new Set();
  privacyPage.on("request", (request) => origins.add(new URL(request.url()).origin));
  for (const route of ["/", "/demo?demo=1", "/privacy", "/terms", "/app"]) {
    await privacyPage.goto(`${baseURL}${route}`, { waitUntil: "networkidle" });
    const serious = (await new AxeBuilder({ page: privacyPage }).analyze()).violations
      .filter((item) => ["serious", "critical"].includes(item.impact ?? "")).length;
    report.axe[route] = serious;
  }
  report.privacyOrigins = [...origins];
  check(report.privacyOrigins.every((origin) => origin === expectedOrigin), "a pre-sign-in page sent a cross-origin request");
  check(Object.values(report.axe).every((count) => count === 0), "axe found a serious or critical issue");
  check(report.consoleErrors.length === 0 && report.pageErrors.length === 0, "browser console or page errors occurred");
  report.offline.serviceWorkers = await privacyPage.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length);
  await privacyContext.setOffline(true);
  try {
    await privacyPage.reload({ waitUntil: "domcontentloaded", timeout: 10_000 });
    report.offline.reload = "loaded";
  } catch {
    report.offline.reload = "unavailable-as-documented";
  }
  check(report.offline.serviceWorkers === 0, "an undocumented service worker is registered");
  await privacyContext.close();

  const identityContext = await browser.newContext();
  const identityPage = await identityContext.newPage();
  await identityPage.goto(`${baseURL}/app`);
  await identityPage.getByRole("button", { name: "Sign in with Sociobot" }).click();
  await identityPage.waitForURL(/sociobotcustomers\.ciamlogin\.com/, { timeout: 30_000 });
  const identityURL = new URL(identityPage.url());
  report.identity = {
    host: identityURL.host,
    tenantInPath: identityURL.pathname.includes("35c6fe40-0ec0-46b6-98c6-213ad4de6650"),
    clientId: identityURL.searchParams.get("client_id"),
    redirectUri: identityURL.searchParams.get("redirect_uri"),
    responseType: identityURL.searchParams.get("response_type"),
    codeChallengeMethod: identityURL.searchParams.get("code_challenge_method")
  };
  check(report.identity.tenantInPath, "identity request used the wrong tenant");
  check(report.identity.clientId === "25c704f4-465a-47af-80ab-2c489466b697", "identity request used the wrong client");
  check(report.identity.redirectUri === `${expectedOrigin}/auth/callback`, "identity request used the wrong callback");
  check(report.identity.responseType === "code" && report.identity.codeChallengeMethod === "S256", "identity request did not use authorization code with PKCE S256");
  await identityContext.close();
} finally {
  await browser.close();
}

report.failures = failures;
await fs.writeFile(path.join(evidenceDir, "browser-smoke.json"), `${JSON.stringify(report, null, 2)}\n`);
if (failures.length) {
  console.error(JSON.stringify(report, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(report, null, 2));
