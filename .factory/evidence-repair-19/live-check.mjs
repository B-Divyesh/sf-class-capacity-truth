import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { writeFile } from "node:fs/promises";

const base = "https://class-capacity-truth.sociobot.in";
const report = {
  checkedAt: new Date().toISOString(),
  implementation: process.env.EXPECTED_BUILD_SHA ?? "not-supplied",
  assertions: [],
  consoleErrors: [],
  pageErrors: [],
};
const runOctet = (Math.floor(Date.now() / 1000) % 200) + 20;
const clientAddress = (scenario) => `198.18.${runOctet}.${scenario}`;
const check = (condition, name, detail = "") => {
  if (!condition) throw new Error(`${name}: ${detail}`);
  report.assertions.push({ name, detail });
};
const watch = (page) => {
  page.on("console", (message) => {
    if (message.type() === "error" && !message.text().includes("status of 404")) {
      report.consoleErrors.push(message.text());
    }
  });
  page.on("pageerror", (error) => report.pageErrors.push(String(error)));
};
const seriousAxe = async (page) => (await new AxeBuilder({ page }).analyze()).violations
  .filter((item) => ["serious", "critical"].includes(item.impact ?? ""));

const browser = await chromium.launch({ headless: true });
try {
  const desktop = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    extraHTTPHeaders: { "x-forwarded-for": clientAddress(1) },
  });
  const page = await desktop.newPage();
  watch(page);
  const requests = [];
  page.on("request", (request) => requests.push({ method: request.method(), url: request.url() }));
  const home = await page.goto(base, { waitUntil: "networkidle" });
  check(home?.status() === 200, "desktop home returns 200", String(home?.status()));
  check(await page.title() === "Class Capacity Truth — Show the right seat count", "home title names the job");
  check(await page.locator("html").getAttribute("lang") === "en", "page language is English");
  check(await page.locator("h1").count() === 1 && await page.locator("main").count() === 1, "home has one h1 and main");
  const headline = page.getByRole("heading", { level: 1, name: "Show the right number of class seats" });
  const audience = page.getByText(/For language schools and tutoring centres/);
  const firstAction = page.getByRole("link", { name: "Try it with sample data" });
  for (const [locator, name] of [[headline, "job"], [audience, "audience"], [firstAction, "first action"]]) {
    const box = await locator.boundingBox();
    check(Boolean(box && box.y >= 0 && box.y + box.height <= 900), `desktop shows the ${name} before scrolling`, JSON.stringify(box));
  }
  await page.screenshot({ path: ".factory/evidence-repair-19/live/home-desktop.png" });

  await firstAction.click();
  await page.getByRole("article").first().waitFor();
  check(new URL(page.url()).pathname === "/demo", "one click opens the demo", page.url());
  check(await page.getByRole("article").count() === 3, "demo loads three realistic classes");
  check(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo label is persistent");
  const openClass = page.getByRole("article").filter({ hasText: "Level check: upper primary" });
  check(await openClass.getByText("2 seats open", { exact: false }).isVisible(), "sample starts with two open seats");
  await openClass.getByRole("link", { name: "Book this sample class" }).click();
  check(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo label stays on the booking form");
  let bookingPosts = 0;
  page.on("request", (request) => {
    if (request.method() === "POST" && /\/api\/demo\/classes\/.+\/book/.test(request.url())) bookingPosts += 1;
  });
  await page.getByLabel("Guardian name").fill("");
  await page.getByLabel("Email address").fill("not-an-email");
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await page.waitForTimeout(250);
  check(bookingPosts === 0, "invalid booking makes no API request", String(bookingPosts));
  check(await page.locator("input:invalid").count() >= 1, "invalid booking identifies invalid input");
  await page.getByLabel("Guardian name").fill("Repair Sample");
  await page.getByLabel("Email address").fill("repair@example.org");
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await page.getByRole("heading", { name: "Your sample seat is booked" }).waitFor();
  check(await page.getByText("1 seat is now open in this class.").isVisible(), "booking updates the visible count");
  check(await page.getByText("Demo — sample data, nothing is saved").isVisible(), "demo label stays on the result");
  await page.screenshot({ path: ".factory/evidence-repair-19/live/booking-success-desktop.png" });
  await page.getByRole("button", { name: "Reset demo" }).click();
  await page.getByRole("article").first().waitFor();
  check(await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "reset restores the sample");

  const full = page.getByRole("article").filter({ hasText: "Friday conversation group" });
  await full.getByRole("link", { name: "View the full class" }).click();
  await page.getByRole("heading", { name: "This class is full" }).waitFor();
  const fullId = new URL(page.url()).pathname.split("/").pop();
  const fullResponse = await page.request.post(`${base}/api/demo/classes/${fullId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID() },
    data: { guardianName: "Repair Sample", guardianEmail: "repair@example.org" },
  });
  check(fullResponse.status() === 409 && (await fullResponse.json()).code === "class_full", "API blocks a full class");
  await page.goto(`${base}/demo?demo=1`);
  await page.getByRole("article").first().waitFor();
  const closed = page.getByRole("article").filter({ hasText: "Saturday assessment" });
  await closed.getByRole("link", { name: "View the closed class" }).click();
  await page.getByRole("heading", { name: "Booking has closed" }).waitFor();
  const closedId = new URL(page.url()).pathname.split("/").pop();
  const closedResponse = await page.request.post(`${base}/api/demo/classes/${closedId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID() },
    data: { guardianName: "Repair Sample", guardianEmail: "repair@example.org" },
  });
  check(closedResponse.status() === 409 && (await closedResponse.json()).code === "booking_closed", "API enforces the booking cutoff");

  await page.setExtraHTTPHeaders({ "x-forwarded-for": clientAddress(2) });
  await page.goto(`${base}/demo?demo=1`);
  await page.getByRole("article").first().waitFor();
  await page.getByRole("button", { name: "Start for real" }).click();
  const realHeading = page.getByRole("heading", { level: 1, name: "Sign in to manage class capacity" });
  await realHeading.waitFor();
  check(await realHeading.evaluate((element) => document.activeElement === element), "Start for real opens and focuses the real workspace");
  check(requests.filter((item) => item.method !== "GET").every((item) => new URL(item.url).pathname.startsWith("/api/demo/")), "demo makes no real-data writes");
  check(requests.every((item) => new URL(item.url).origin === base), "demo stays same-origin");

  const isolated = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": clientAddress(3) } });
  const isolatedPage = await isolated.newPage();
  await isolatedPage.goto(`${base}/demo?demo=1`);
  await isolatedPage.getByRole("article").first().waitFor();
  check(await isolatedPage.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false }).isVisible(), "a second browser gets an isolated sample");
  await isolated.close();
  await desktop.close();

  const phone = await browser.newContext({
    viewport: { width: 390, height: 844 },
    colorScheme: "dark",
    reducedMotion: "reduce",
    extraHTTPHeaders: { "x-forwarded-for": clientAddress(4) },
  });
  const mobile = await phone.newPage();
  watch(mobile);
  await mobile.goto(base, { waitUntil: "networkidle" });
  check(await mobile.evaluate(() => scrollY === 0), "fresh phone starts before scrolling");
  for (const [locator, name] of [
    [mobile.getByRole("heading", { level: 1, name: "Show the right number of class seats" }), "job"],
    [mobile.getByText(/For language schools and tutoring centres/), "audience"],
    [mobile.getByRole("link", { name: "Try it with sample data" }), "first action"],
  ]) {
    const box = await locator.boundingBox();
    check(Boolean(box && box.y >= 0 && box.y + box.height <= 844), `phone shows the ${name} before scrolling`, JSON.stringify(box));
  }
  check(await mobile.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "phone home has no horizontal overflow");
  await mobile.screenshot({ path: ".factory/evidence-repair-19/live/home-phone-dark-reduced.png" });
  await mobile.keyboard.press("Tab");
  check(await mobile.getByRole("link", { name: "Skip to main content" }).evaluate((element) => document.activeElement === element), "skip link is first in keyboard order");
  await mobile.keyboard.press("Enter");
  check(await mobile.locator("main").evaluate((element) => document.activeElement === element), "skip link focuses main");
  const menu = mobile.getByRole("button", { name: "Open main menu" });
  const menuBox = await menu.boundingBox();
  check(Boolean(menuBox && menuBox.width >= 44 && menuBox.height >= 44), "phone menu meets the touch-target size", JSON.stringify(menuBox));
  await menu.focus();
  await mobile.keyboard.press("Enter");
  check(await mobile.getByRole("navigation", { name: "Main navigation" }).isVisible(), "phone menu opens by keyboard");
  await mobile.keyboard.press("Escape");
  check(await menu.evaluate((element) => document.activeElement === element), "phone menu restores focus after Escape");
  const moving = await mobile.locator("*").evaluateAll((items) => items.filter((item) => {
    const style = getComputedStyle(item);
    return style.animationDuration !== "0s" || style.transitionDuration.split(",").some((part) => part.trim() !== "0s");
  }).length);
  check(moving === 0, "reduced motion removes animation and transition durations", String(moving));
  await mobile.goto(`${base}/demo?demo=1`);
  await mobile.getByRole("article").first().waitFor();
  await mobile.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  check(await mobile.getByRole("article").count() === 3, "phone demo works at 200 percent text");
  check(await mobile.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "phone demo reflows at 200 percent text");
  await mobile.screenshot({ path: ".factory/evidence-repair-19/live/demo-phone-dark-reduced-200.png", fullPage: true });
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
    ["/repair-19-deliberate-missing", 404, "Page not found — Class Capacity Truth"],
  ];
  const sweepContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": clientAddress(5) } });
  const sweep = await sweepContext.newPage();
  watch(sweep);
  for (const [path, status, title] of routes) {
    const response = await sweep.goto(`${base}${path}`);
    check(response?.status() === status, `${path} returns ${status}`, String(response?.status()));
    check(await sweep.title() === title, `${path} has the route title`, await sweep.title());
    check(await sweep.locator("h1").count() === 1 && await sweep.locator("main").count() === 1, `${path} has one h1 and main`);
    const violations = await seriousAxe(sweep);
    check(violations.length === 0, `${path} has no serious or critical Axe issue`, violations.map((item) => item.id).join(","));
  }
  check(await sweep.getByRole("link", { name: "Privacy" }).isVisible() && await sweep.getByRole("link", { name: "Terms" }).isVisible(), "404 retains legal links");
  await sweepContext.close();

  check(report.consoleErrors.length === 0, "checked pages log no unexpected console errors", report.consoleErrors.join(" | "));
  check(report.pageErrors.length === 0, "checked pages raise no page errors", report.pageErrors.join(" | "));
  report.verdict = "PASS";
} finally {
  await browser.close();
  await writeFile(".factory/evidence-repair-19/live/browser-check.json", `${JSON.stringify(report, null, 2)}\n`);
}
