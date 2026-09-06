import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { writeFile } from "node:fs/promises";

const base = "https://class-capacity-truth.sociobot.in";
const evidence = { checkedAt: new Date().toISOString(), base, assertions: [], consoleErrors: [], pageErrors: [] };
const assert = (condition, name, detail = "") => {
  if (!condition) throw new Error(`${name}: ${detail}`);
  evidence.assertions.push({ name, detail });
};
const attachErrors = (page) => {
  page.on("console", (message) => { if (message.type() === "error") evidence.consoleErrors.push(message.text()); });
  page.on("pageerror", (error) => evidence.pageErrors.push(String(error)));
};
const seriousAxe = async (page) => (await new AxeBuilder({ page }).analyze()).violations
  .filter((item) => ["serious", "critical"].includes(item.impact ?? ""));

const browser = await chromium.launch({ headless: true });
try {
  const desktop = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    extraHTTPHeaders: { "x-forwarded-for": "198.51.100.203" }
  });
  const page = await desktop.newPage();
  attachErrors(page);
  const requests = [];
  page.on("request", (request) => requests.push({ method: request.method(), url: request.url() }));
  const home = await page.goto(base, { waitUntil: "networkidle" });
  assert(home?.status() === 200, "desktop home returns 200", String(home?.status()));
  assert(await page.title() === "Class Capacity Truth — Show the right seat count", "home title names the job");
  assert(await page.locator("html").getAttribute("lang") === "en", "html language is set");
  assert(await page.locator("h1").count() === 1 && await page.locator("main").count() === 1, "home has one h1 and main");
  assert(await page.getByRole("heading", { level: 1, name: "Show the right number of class seats" }).isVisible(), "desktop first screen states the job");
  assert(await page.getByText(/For language schools and tutoring centres/).isVisible(), "desktop first screen states the audience");
  const primary = page.getByRole("link", { name: "Try it with sample data" });
  assert(await primary.isVisible(), "desktop first screen shows the first action");
  assert(await page.getByText("See three sample classes next.").isVisible(), "desktop first action states its result");
  for (const locator of [page.locator("h1"), page.getByText(/For language schools and tutoring centres/), primary]) {
    const box = await locator.boundingBox();
    assert(Boolean(box && box.y >= 0 && box.y + box.height <= 900), "desktop first-read item is above the fold", JSON.stringify(box));
  }
  await page.screenshot({ path: ".factory/review-3-evidence/home-desktop.png", fullPage: false });
  await primary.click();
  await page.getByRole("article").first().waitFor({ state: "visible" });
  assert(new URL(page.url()).pathname === "/demo", "one click opens the demo", page.url());
  assert(await page.getByRole("article").count() === 3, "demo loads three realistic classes");
  assert(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "persistent demo label is visible");
  const names = await page.getByRole("article").locator("h2").allTextContents();
  assert(names.join("|") === "Level check: upper primary|Friday conversation group|Saturday assessment", "sample names are realistic", names.join("|"));
  const openClass = page.getByRole("article").filter({ hasText: "Level check: upper primary" });
  assert(await openClass.getByText("2 seats open", { exact: false }).isVisible(), "sample begins with two open seats");
  await openClass.getByRole("link", { name: "Book this sample class" }).click();
  assert(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo label persists on booking form");
  let bookingPosts = 0;
  page.on("request", (request) => { if (request.method() === "POST" && /\/api\/demo\/classes\/.+\/book/.test(request.url())) bookingPosts++; });
  await page.getByLabel("Guardian name").fill("");
  await page.getByLabel("Guardian email").fill("not-an-email");
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await page.waitForTimeout(250);
  assert(bookingPosts === 0, "invalid form sends no booking request", String(bookingPosts));
  assert(await page.locator("input:invalid").count() >= 1, "invalid form exposes native validation");
  await page.getByLabel("Guardian name").fill("Review Sample");
  await page.getByLabel("Guardian email").fill("review@example.org");
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  assert(await page.getByRole("heading", { name: "Your sample seat is booked" }).isVisible(), "corrected form books a sample seat");
  assert(await page.getByText("1 seat is now open in this class.").isVisible(), "booking updates the visible count");
  assert(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo label persists on booking result");
  await page.screenshot({ path: ".factory/review-3-evidence/booking-success-desktop.png", fullPage: false });
  await page.getByRole("button", { name: "Reset demo" }).click();
  await page.getByRole("article").first().waitFor({ state: "visible" });
  assert(await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "reset restores two open seats");

  const full = page.getByRole("article").filter({ hasText: "Friday conversation group" });
  await full.getByRole("link", { name: "View the full class" }).click();
  assert(await page.getByRole("heading", { name: "This class is full" }).isVisible(), "full boundary explains the state");
  assert(await page.getByRole("button", { name: "Book one sample seat" }).count() === 0, "full class has no booking action");
  const fullId = new URL(page.url()).pathname.split("/").pop();
  const fullResponse = await page.request.post(`${base}/api/demo/classes/${fullId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID(), "x-forwarded-for": "198.51.100.203" },
    data: { guardianName: "Review Sample", guardianEmail: "review@example.org" }
  });
  assert(fullResponse.status() === 409 && (await fullResponse.json()).code === "class_full", "full boundary is enforced by the API");
  await page.goto(`${base}/demo?demo=1`);
  await page.getByRole("article").first().waitFor({ state: "visible" });
  const closed = page.getByRole("article").filter({ hasText: "Saturday assessment" });
  await closed.getByRole("link", { name: "View the closed class" }).click();
  assert(await page.getByRole("heading", { name: "Booking has closed" }).isVisible(), "cutoff boundary explains the state");
  const closedId = new URL(page.url()).pathname.split("/").pop();
  const closedResponse = await page.request.post(`${base}/api/demo/classes/${closedId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID(), "x-forwarded-for": "198.51.100.203" },
    data: { guardianName: "Review Sample", guardianEmail: "review@example.org" }
  });
  assert(closedResponse.status() === 409 && (await closedResponse.json()).code === "booking_closed", "cutoff boundary is enforced by the API");

  await page.goto(`${base}/demo?demo=1`);
  await page.getByRole("article").first().waitFor({ state: "visible" });
  await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await page.getByRole("button", { name: "Start for real" }).click();
  assert(new URL(page.url()).pathname === "/app", "Start for real opens the real workspace", page.url());
  const realHeading = page.getByRole("heading", { level: 1, name: "Create your school workspace" });
  assert(await realHeading.isFocused(), "Start for real focuses the workspace heading");
  await page.goto(`${base}/demo?demo=1`);
  await page.getByRole("article").first().waitFor({ state: "visible" });
  assert(await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "Start for real destroys the prior demo");
  const nonGet = requests.filter((item) => item.method !== "GET").map((item) => new URL(item.url).pathname);
  assert(nonGet.every((path) => path.startsWith("/api/demo/")), "demo flow makes no real-data writes", nonGet.join(","));
  assert(requests.every((item) => new URL(item.url).origin === base), "public and demo flow stays same-origin");
  assert((await page.evaluate(() => navigator.serviceWorker?.getRegistrations().then((items) => items.length) ?? 0)) === 0, "no service worker is registered");

  const isolated = await browser.newContext({ viewport: { width: 1440, height: 900 }, extraHTTPHeaders: { "x-forwarded-for": "198.51.100.204" } });
  const isolatedPage = await isolated.newPage();
  await isolatedPage.goto(`${base}/demo?demo=1`);
  await isolatedPage.getByRole("article").first().waitFor({ state: "visible" });
  assert(await isolatedPage.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "a second browser has an isolated seed");
  await isolated.close();
  await desktop.close();

  const phone = await browser.newContext({
    viewport: { width: 390, height: 844 },
    colorScheme: "dark",
    reducedMotion: "reduce",
    extraHTTPHeaders: { "x-forwarded-for": "198.51.100.205" }
  });
  const mobile = await phone.newPage();
  attachErrors(mobile);
  await mobile.goto(base, { waitUntil: "networkidle" });
  assert(await mobile.evaluate(() => scrollY === 0), "phone first read starts before scrolling");
  assert(await mobile.getByRole("heading", { level: 1, name: "Show the right number of class seats" }).isVisible(), "phone first screen states the job");
  assert(await mobile.getByText(/For language schools and tutoring centres/).isVisible(), "phone first screen states the audience");
  assert(await mobile.getByRole("link", { name: "Try it with sample data" }).isVisible(), "phone first screen shows the first action");
  for (const locator of [mobile.locator("h1"), mobile.getByText(/For language schools and tutoring centres/), mobile.getByRole("link", { name: "Try it with sample data" })]) {
    const box = await locator.boundingBox();
    assert(Boolean(box && box.y >= 0 && box.y + box.height <= 844), "phone first-read item is above the fold", JSON.stringify(box));
  }
  assert(await mobile.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "phone home has no horizontal overflow");
  await mobile.screenshot({ path: ".factory/review-3-evidence/home-phone-dark-reduced.png", fullPage: false });
  await mobile.keyboard.press("Tab");
  assert(await mobile.getByRole("link", { name: "Skip to main content" }).isFocused(), "skip link is first in keyboard order");
  await mobile.keyboard.press("Enter");
  assert(await mobile.locator("main").isFocused(), "skip link moves focus to main");
  const menu = mobile.getByRole("button", { name: "Open main menu" });
  const menuBox = await menu.boundingBox();
  assert(Boolean(menuBox && menuBox.width >= 44 && menuBox.height >= 44), "phone menu meets the touch target", JSON.stringify(menuBox));
  await menu.focus();
  await mobile.keyboard.press("Enter");
  assert(await mobile.getByRole("navigation", { name: "Main navigation" }).isVisible(), "phone menu opens by keyboard");
  await mobile.keyboard.press("Escape");
  assert(await menu.isFocused(), "phone menu restores focus after Escape");
  const moving = await mobile.locator("*").evaluateAll((items) => items.filter((item) => {
    const style = getComputedStyle(item);
    return style.animationDuration !== "0s" || style.transitionDuration.split(",").some((part) => part.trim() !== "0s");
  }).length);
  assert(moving === 0, "reduced motion removes animation and transition duration", String(moving));
  await mobile.goto(`${base}/demo?demo=1`);
  await mobile.getByRole("article").first().waitFor({ state: "visible" });
  await mobile.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  assert(await mobile.getByRole("article").count() === 3, "phone demo loads at 200 percent text");
  assert(await mobile.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "phone demo reflows at 200 percent text");
  await phone.close();

  const routes = [
    ["/", 200, "Class Capacity Truth — Show the right seat count"],
    ["/demo?demo=1", 200, "Demo — Class Capacity Truth"],
    ["/app", 200, "Classes — Class Capacity Truth"],
    ["/app/reconciliation", 200, "Calendar checks — Class Capacity Truth"],
    ["/app/waitlist", 200, "Waitlist offers — Class Capacity Truth"],
    ["/app/settings", 200, "Settings — Class Capacity Truth"],
    ["/app/settings/billing", 200, "Billing — Class Capacity Truth"],
    ["/app/settings/data", 200, "School data — Class Capacity Truth"],
    ["/app/operations", 200, "Operations — Class Capacity Truth"],
    ["/privacy", 200, "Privacy — Class Capacity Truth"],
    ["/terms", 200, "Terms — Class Capacity Truth"],
    ["/review-3-deliberate-missing", 404, "Page not found — Class Capacity Truth"]
  ];
  const sweepContext = await browser.newContext({ viewport: { width: 1280, height: 900 }, extraHTTPHeaders: { "x-forwarded-for": "198.51.100.206" } });
  const sweep = await sweepContext.newPage();
  attachErrors(sweep);
  for (const [path, status, title] of routes) {
    const response = await sweep.goto(`${base}${path}`);
    assert(response?.status() === status, `${path} returns ${status}`, String(response?.status()));
    assert(await sweep.title() === title, `${path} has the route title`, await sweep.title());
    assert(await sweep.locator("h1").count() === 1 && await sweep.locator("main").count() === 1, `${path} has one h1 and main`);
    const violations = await seriousAxe(sweep);
    assert(violations.length === 0, `${path} has no serious or critical Axe issue`, violations.map((item) => item.id).join(","));
  }
  const missingResponse = await sweep.goto(`${base}/review-3-deliberate-missing`);
  assert(missingResponse?.status() === 404, "unknown route is a deliberate HTTP 404");
  assert(await sweep.getByRole("link", { name: "Privacy" }).isVisible() && await sweep.getByRole("link", { name: "Terms" }).isVisible(), "404 retains legal links");
  assert(Boolean(await sweep.locator('meta[name="description"]').getAttribute("content")), "404 retains metadata");
  await sweep.setViewportSize({ width: 390, height: 844 });
  await sweep.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  assert(await sweep.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "404 reflows at phone width and 200 percent text");
  const recoveryBox = await sweep.getByRole("link", { name: "Go to Class Capacity Truth" }).boundingBox();
  assert(Boolean(recoveryBox && recoveryBox.height >= 44), "404 recovery link meets the touch target", JSON.stringify(recoveryBox));
  await sweepContext.close();

  assert(evidence.consoleErrors.length === 0, "live browser console has no errors", evidence.consoleErrors.join(" | "));
  assert(evidence.pageErrors.length === 0, "live browser has no page errors", evidence.pageErrors.join(" | "));
  evidence.verdict = "PASS";
} finally {
  await browser.close();
  await writeFile(".factory/review-3-evidence/live-review.json", `${JSON.stringify(evidence, null, 2)}\n`);
}
