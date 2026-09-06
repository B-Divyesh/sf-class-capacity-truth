import fs from "node:fs/promises";
import path from "node:path";
import AxeBuilder from "@axe-core/playwright";
import { chromium, request } from "@playwright/test";

const baseURL = "https://class-capacity-truth.sociobot.in";
const outDir = ".factory/review-4-evidence";
const failures = [];
const report = {
  checkedAt: new Date().toISOString(),
  baseURL,
  firstRead: {},
  demo: {},
  accessibility: {},
  routes: {},
  privacy: {},
  identity: {},
  links: [],
  consoleErrors: [],
  pageErrors: []
};
const check = (condition, message) => { if (!condition) failures.push(message); };
const serious = (result) => result.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""));

await fs.mkdir(outDir, { recursive: true });
const browser = await chromium.launch();
try {
  for (const [name, viewport] of Object.entries({ desktop: { width: 1440, height: 900 }, phone: { width: 390, height: 844 } })) {
    const context = await browser.newContext({ viewport, colorScheme: name === "phone" ? "dark" : "light", reducedMotion: name === "phone" ? "reduce" : "no-preference", extraHTTPHeaders: { "x-forwarded-for": name === "phone" ? "198.51.100.184" : "198.51.100.183" } });
    const page = await context.newPage();
    page.on("console", (message) => { if (message.type() === "error") report.consoleErrors.push(`${name}: ${message.text()}`); });
    page.on("pageerror", (error) => report.pageErrors.push(`${name}: ${String(error)}`));
    await page.goto(baseURL, { waitUntil: "networkidle" });
    const first = await page.evaluate(() => {
      const h1 = document.querySelector("h1");
      const audience = [...document.querySelectorAll("p")].find((node) => node.textContent?.includes("language schools and tutoring centres"));
      const action = [...document.querySelectorAll("a")].find((node) => node.textContent?.includes("Try it with sample data"));
      const result = [...document.querySelectorAll("p, span")].find((node) => node.textContent?.trim() === "See three sample classes next.");
      const visible = (node) => Boolean(node && node.getBoundingClientRect().top >= 0 && node.getBoundingClientRect().bottom <= innerHeight);
      return { title: document.title, h1: h1?.textContent?.trim(), audience: audience?.textContent?.trim(), action: action?.textContent?.trim(), result: result?.textContent?.trim(), beforeScroll: [h1, audience, action, result].every(visible), h1Count: document.querySelectorAll("h1").length, mainCount: document.querySelectorAll("main").length };
    });
    report.firstRead[name] = first;
    check(first.h1 === "Show the right number of class seats", `${name} job heading is wrong`);
    check(first.audience === "For language schools and tutoring centres", `${name} audience is missing`);
    check(first.action === "Try it with sample data", `${name} first action is missing`);
    check(first.result === "See three sample classes next.", `${name} action result is missing`);
    check(first.beforeScroll, `${name} first-read content is not all visible before scrolling`);
    check(first.h1Count === 1 && first.mainCount === 1, `${name} home semantics failed`);
    await page.screenshot({ path: path.join(outDir, `first-read-${name}.png`), fullPage: false });
    await context.close();
  }

  const contextA = await browser.newContext({ viewport: { width: 1440, height: 900 }, extraHTTPHeaders: { "x-forwarded-for": "198.51.100.185" } });
  const pageA = await contextA.newPage();
  const requests = [];
  pageA.on("request", (req) => requests.push({ method: req.method(), url: req.url() }));
  pageA.on("console", (message) => { if (message.type() === "error") report.consoleErrors.push(`demo: ${message.text()}`); });
  pageA.on("pageerror", (error) => report.pageErrors.push(`demo: ${String(error)}`));
  await pageA.goto(baseURL, { waitUntil: "networkidle" });
  await pageA.getByRole("link", { name: "Try it with sample data" }).click();
  await pageA.getByRole("article").first().waitFor();
  const articles = await pageA.getByRole("article").count();
  const banner = pageA.getByText("Demo — sample data, nothing is saved");
  check(articles === 3, "one-click demo did not load three classes");
  check(await banner.isVisible(), "demo label is missing");
  const openCard = pageA.getByRole("article").filter({ hasText: "Level check: upper primary" });
  check(await openCard.getByText("2 seats open", { exact: false }).isVisible(), "open sample did not start with two seats");
  await openCard.getByRole("link", { name: "Book this sample class" }).click();
  let bookingPosts = 0;
  pageA.on("request", (requestItem) => { if (requestItem.method() === "POST" && requestItem.url().includes("/book")) bookingPosts += 1; });
  await pageA.getByLabel("Guardian name").fill("");
  await pageA.getByLabel("Email address").fill("not-an-email");
  await pageA.getByRole("button", { name: "Book one sample seat" }).click();
  await pageA.waitForTimeout(250);
  const invalid = await pageA.locator("input:invalid").count();
  check(invalid === 2 && bookingPosts === 0, "invalid sample input was not blocked locally");
  await pageA.getByLabel("Guardian name").fill("Alex Morgan");
  await pageA.getByLabel("Email address").fill("alex@example.org");
  await pageA.getByRole("button", { name: "Book one sample seat" }).click();
  await pageA.getByText("1 seat is now open in this class.").waitFor();
  check(await banner.isVisible(), "demo label did not persist through booking");
  await pageA.screenshot({ path: path.join(outDir, "booking-success-desktop.png"), fullPage: true });
  await pageA.getByRole("button", { name: "Reset demo" }).click();
  await pageA.getByRole("article").first().waitFor();
  check(await pageA.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "reset did not restore two seats");

  const fullCard = pageA.getByRole("article").filter({ hasText: "Friday conversation group" });
  await fullCard.getByRole("link", { name: "View the full class" }).click();
  await pageA.getByRole("heading", { name: "This class is full" }).waitFor();
  const fullId = new URL(pageA.url()).pathname.split("/").pop();
  const fullResponse = await pageA.request.post(`${baseURL}/api/demo/classes/${fullId}/book`, { headers: { "Idempotency-Key": crypto.randomUUID() }, data: { guardianName: "Alex Morgan", guardianEmail: "alex@example.org" } });
  check(fullResponse.status() === 409 && (await fullResponse.json()).code === "class_full", "full-class boundary did not return class_full");
  const cutoffContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.186" } });
  const cutoffPage = await cutoffContext.newPage();
  await cutoffPage.goto(`${baseURL}/demo?demo=1`, { waitUntil: "networkidle" });
  const cutoffCard = cutoffPage.getByRole("article").filter({ hasText: "Saturday assessment" });
  await cutoffCard.getByRole("link", { name: "View the closed class" }).click();
  await cutoffPage.getByRole("heading", { name: "Booking has closed" }).waitFor();
  const cutoffId = new URL(cutoffPage.url()).pathname.split("/").pop();
  const cutoffResponse = await cutoffPage.request.post(`${baseURL}/api/demo/classes/${cutoffId}/book`, { headers: { "Idempotency-Key": crypto.randomUUID() }, data: { guardianName: "Alex Morgan", guardianEmail: "alex@example.org" } });
  check(cutoffResponse.status() === 409 && (await cutoffResponse.json()).code === "booking_closed", "cutoff boundary did not return booking_closed");
  await cutoffContext.close();
  const contextB = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.187" } });
  const pageB = await contextB.newPage();
  await pageB.goto(`${baseURL}/demo?demo=1`, { waitUntil: "networkidle" });
  check(await pageB.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "second browser did not receive isolated sample data");
  await contextB.close();

  const contextC = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.188" } });
  const pageC = await contextC.newPage();
  const exitRequests = [];
  pageC.on("request", (req) => exitRequests.push({ method: req.method(), url: req.url() }));
  await pageC.goto(`${baseURL}/demo?demo=1`, { waitUntil: "networkidle" });
  await pageC.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await pageC.getByLabel("Guardian name").fill("Alex Morgan");
  await pageC.getByLabel("Email address").fill("alex@example.org");
  await pageC.getByRole("button", { name: "Book one sample seat" }).click();
  await pageC.getByText("1 seat is now open in this class.").waitFor();
  await pageC.getByRole("button", { name: "Start for real" }).click();
  await pageC.waitForURL(`${baseURL}/app`);
  await pageC.getByRole("heading", { level: 1, name: "Sign in to manage class capacity" }).waitFor();
  const realHeading = await pageC.getByRole("heading", { level: 1 }).textContent();
  check(realHeading?.includes("Sign in") === true, "Start for real did not reach the signed-out workspace");
  const writes = [...requests, ...exitRequests].filter((item) => item.method !== "GET" && item.method !== "HEAD");
  check(writes.every((item) => new URL(item.url).pathname.startsWith("/api/demo")), "demo flow wrote outside demo endpoints");
  report.demo = { articles, invalidFields: invalid, bookingPostsAfterInvalid: 0, validSeatResult: "2 to 1", resetSeatResult: 2, fullStatus: fullResponse.status(), cutoffStatus: cutoffResponse.status(), secondBrowserSeats: 2, startForRealHeading: realHeading, nonGetRequests: writes };
  await contextA.close();
  await contextC.close();

  const accessibilityContext = await browser.newContext({ viewport: { width: 390, height: 844 }, colorScheme: "dark", reducedMotion: "reduce", extraHTTPHeaders: { "x-forwarded-for": "198.51.100.189" } });
  const mobile = await accessibilityContext.newPage();
  await mobile.goto(baseURL, { waitUntil: "networkidle" });
  await mobile.keyboard.press("Tab");
  check(await mobile.getByRole("link", { name: "Skip to main content" }).evaluate((node) => node === document.activeElement), "skip link was not first in keyboard order");
  await mobile.keyboard.press("Enter");
  check(await mobile.locator("main").evaluate((node) => node === document.activeElement), "skip link did not focus main");
  const menu = mobile.getByRole("button", { name: "Open main menu" });
  const menuBox = await menu.boundingBox();
  await menu.focus();
  await mobile.keyboard.press("Enter");
  check(await mobile.getByRole("navigation", { name: "Main navigation" }).isVisible(), "phone menu did not open with Enter");
  await mobile.keyboard.press("Escape");
  check(await menu.evaluate((node) => node === document.activeElement), "phone menu did not restore focus after Escape");
  const motion = await mobile.evaluate(() => ({ animations: [...new Set([...document.querySelectorAll("*")].flatMap((node) => getComputedStyle(node).animationDuration.split(",")))], transitions: [...new Set([...document.querySelectorAll("*")].flatMap((node) => getComputedStyle(node).transitionDuration.split(",")))] }));
  check(Boolean(menuBox && menuBox.width >= 44 && menuBox.height >= 44), "phone menu target is under 44px");
  check(motion.animations.every((value) => value.trim() === "0s") && motion.transitions.every((value) => value.trim() === "0s"), "reduced motion left motion enabled");

  const routeExpectations = [
    ["/", "Class Capacity Truth — Show the right seat count"],
    ["/demo?demo=1", "Demo — Class Capacity Truth"],
    ["/app", "Classes — Class Capacity Truth"],
    ["/app/reconciliation", "Calendar checks — Class Capacity Truth"],
    ["/app/waitlist", "Waitlist offers — Class Capacity Truth"],
    ["/app/settings", "Settings — Class Capacity Truth"],
    ["/app/settings/billing", "Billing — Class Capacity Truth"],
    ["/app/settings/data", "School data — Class Capacity Truth"],
    ["/app/operations", "Operations — Class Capacity Truth"],
    ["/privacy", "Privacy — Class Capacity Truth"],
    ["/terms", "Terms — Class Capacity Truth"],
    ["/not-a-real-page", "Page not found — Class Capacity Truth"]
  ];
  for (const [route, title] of routeExpectations) {
    const response = await mobile.goto(`${baseURL}${route}`, { waitUntil: "networkidle" });
    if (route.includes("demo")) await mobile.getByRole("article").first().waitFor();
    await mobile.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
    const axeResult = await new AxeBuilder({ page: mobile }).analyze();
    const routeData = await mobile.evaluate(() => ({ title: document.title, h1: document.querySelectorAll("h1").length, main: document.querySelectorAll("main").length, overflow: document.documentElement.scrollWidth > document.documentElement.clientWidth, heading: document.querySelector("h1")?.textContent?.trim() }));
    routeData.status = response?.status();
    routeData.seriousOrCritical = serious(axeResult).map((item) => item.id);
    report.routes[route] = routeData;
    check(routeData.title === title, `${route} title is wrong`);
    check(routeData.h1 === 1 && routeData.main === 1 && !routeData.overflow, `${route} structure or 200% reflow failed`);
    check(routeData.seriousOrCritical.length === 0, `${route} has serious/critical Axe findings`);
    if (route === "/not-a-real-page") check(response?.status() === 404, "unknown route did not return deliberate HTTP 404");
    await mobile.evaluate(() => { document.documentElement.style.fontSize = "100%"; });
  }
  report.accessibility = { menuBox, motion };
  await mobile.screenshot({ path: path.join(outDir, "phone-dark-reduced-200.png"), fullPage: true });
  await accessibilityContext.close();

  const privacyContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.190" } });
  const privacyPage = await privacyContext.newPage();
  const origins = new Set();
  privacyPage.on("request", (req) => origins.add(new URL(req.url()).origin));
  for (const route of ["/", "/demo?demo=1", "/privacy", "/terms", "/app"]) {
    await privacyPage.goto(`${baseURL}${route}`, { waitUntil: "networkidle" });
    if (route.includes("demo")) await privacyPage.getByRole("article").first().waitFor();
  }
  const registrations = await privacyPage.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length);
  report.privacy = { origins: [...origins], serviceWorkers: registrations };
  check([...origins].every((origin) => origin === baseURL), "a public route contacted a third-party origin before explicit action");
  check(registrations === 0, "an undocumented service worker is registered");
  await privacyPage.goto(`${baseURL}/app`);
  await privacyPage.getByRole("button", { name: "Sign in with Sociobot" }).click();
  await privacyPage.waitForURL(/sociobotcustomers\.ciamlogin\.com/, { timeout: 30_000 });
  const identityURL = new URL(privacyPage.url());
  report.identity = { host: identityURL.host, tenant: identityURL.pathname.includes("35c6fe40-0ec0-46b6-98c6-213ad4de6650"), clientId: identityURL.searchParams.get("client_id"), redirectUri: identityURL.searchParams.get("redirect_uri"), responseType: identityURL.searchParams.get("response_type"), codeChallengeMethod: identityURL.searchParams.get("code_challenge_method") };
  check(report.identity.tenant && report.identity.clientId === "25c704f4-465a-47af-80ab-2c489466b697", "CIAM tenant or client is wrong");
  check(report.identity.redirectUri === `${baseURL}/auth/callback` && report.identity.responseType === "code" && report.identity.codeChallengeMethod === "S256", "CIAM callback or PKCE is wrong");
  await privacyContext.close();

  const api = await request.newContext();
  const linkPageContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.191" } });
  const linkPage = await linkPageContext.newPage();
  await linkPage.goto(baseURL, { waitUntil: "networkidle" });
  const hrefs = await linkPage.locator("a").evaluateAll((items) => [...new Set(items.map((item) => item.href).filter(Boolean))]);
  for (const href of hrefs) {
    if (href.startsWith("mailto:")) { report.links.push({ href, status: "mailto" }); continue; }
    const response = await api.get(href, { maxRedirects: 5 });
    report.links.push({ href, status: response.status() });
    check(response.status() >= 200 && response.status() < 400, `link failed: ${href} (${response.status()})`);
  }
  await api.dispose();
  await linkPageContext.close();
} finally {
  await browser.close();
}

report.failures = failures;
await fs.writeFile(path.join(outDir, "live-review.json"), `${JSON.stringify(report, null, 2)}\n`);
if (failures.length) {
  console.error(JSON.stringify(report, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(report, null, 2));
