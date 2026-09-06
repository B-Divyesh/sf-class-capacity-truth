import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { writeFile } from "node:fs/promises";

const base = "https://class-capacity-truth.sociobot.in";
const result = { checkedAt: new Date().toISOString(), assertions: [], consoleErrors: [], pageErrors: [] };
const check = (condition, name, detail = "") => {
  result.assertions.push({ name, pass: Boolean(condition), detail });
  if (!condition) throw new Error(`${name}: ${detail}`);
};
const watch = (page) => {
  page.on("console", (message) => { if (message.type() === "error") result.consoleErrors.push(message.text()); });
  page.on("pageerror", (error) => result.pageErrors.push(String(error)));
};
const browser = await chromium.launch({ headless: true });
try {
  const phoneContext = await browser.newContext({
    viewport: { width: 390, height: 844 }, colorScheme: "dark", reducedMotion: "reduce",
    extraHTTPHeaders: { "x-forwarded-for": "203.0.113.220" }
  });
  const phone = await phoneContext.newPage();
  watch(phone);
  await phone.goto(base, { waitUntil: "networkidle" });
  check(await phone.evaluate(() => scrollY === 0), "fresh phone starts before scrolling");
  check(await phone.getByRole("heading", { level: 1, name: "Show the right number of class seats" }).isVisible(), "phone states the job");
  check(await phone.getByText(/For language schools and tutoring centres/).isVisible(), "phone states the audience");
  check(await phone.getByRole("link", { name: "Try it with sample data" }).isVisible(), "phone states the first action");
  for (const locator of [phone.locator("h1"), phone.getByText(/For language schools and tutoring centres/), phone.getByRole("link", { name: "Try it with sample data" })]) {
    const box = await locator.boundingBox();
    check(Boolean(box && box.y >= 0 && box.y + box.height <= 844), "phone first-read item is above the fold", JSON.stringify(box));
  }
  check(await phone.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "phone home has no horizontal overflow");
  await phone.screenshot({ path: ".factory/review-3-evidence/home-phone-dark-reduced.png", fullPage: false });
  await phone.keyboard.press("Tab");
  check(await phone.getByRole("link", { name: "Skip to main content" }).evaluate((element) => document.activeElement === element), "skip link is first in keyboard order");
  await phone.keyboard.press("Enter");
  check(await phone.locator("main").evaluate((element) => document.activeElement === element), "skip link focuses main");
  const menu = phone.getByRole("button", { name: "Open main menu" });
  const box = await menu.boundingBox();
  check(Boolean(box && box.width >= 44 && box.height >= 44), "mobile menu is at least 44 by 44 pixels", JSON.stringify(box));
  await menu.focus();
  await phone.keyboard.press("Enter");
  check(await phone.getByRole("navigation", { name: "Main navigation" }).isVisible(), "mobile menu opens by keyboard");
  await phone.keyboard.press("Escape");
  check(await menu.evaluate((element) => document.activeElement === element), "mobile menu closes and restores focus");
  const moving = await phone.locator("*").evaluateAll((items) => items.filter((item) => {
    const style = getComputedStyle(item);
    return style.animationDuration !== "0s" || style.transitionDuration.split(",").some((part) => part.trim() !== "0s");
  }).length);
  check(moving === 0, "reduced motion removes animation and transition durations", String(moving));
  const phoneAxe = (await new AxeBuilder({ page: phone }).analyze()).violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""));
  check(phoneAxe.length === 0, "phone dark reduced-motion home has no serious or critical Axe issue", phoneAxe.map((item) => item.id).join(","));
  await phoneContext.close();

  const desktopContext = await browser.newContext({ viewport: { width: 1440, height: 900 }, extraHTTPHeaders: { "x-forwarded-for": "203.0.113.221" } });
  const page = await desktopContext.newPage();
  watch(page);
  const origins = [];
  page.on("request", (request) => origins.push(new URL(request.url()).origin));
  const routes = [
    ["/", 200, "Class Capacity Truth — Show the right seat count"],
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
  for (const [path, status, title] of routes) {
    const response = await page.goto(`${base}${path}`);
    check(response?.status() === status, `${path} returns ${status}`, String(response?.status()));
    check(await page.title() === title, `${path} has its route title`, await page.title());
    check(await page.locator("h1").count() === 1 && await page.locator("main").count() === 1, `${path} has one h1 and main`);
    const violations = (await new AxeBuilder({ page }).analyze()).violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""));
    check(violations.length === 0, `${path} has no serious or critical Axe issue`, violations.map((item) => item.id).join(","));
  }
  check(origins.every((origin) => origin === base), "public, legal, and signed-out routes remain same-origin before an explicit action", [...new Set(origins)].join(","));
  const unexpectedConsoleErrors = result.consoleErrors.filter((message) => !message.includes("status of 404"));
  check(unexpectedConsoleErrors.length === 0, "checked non-demo routes log no unexpected console errors", unexpectedConsoleErrors.join(" | "));
  check(result.pageErrors.length === 0, "checked non-demo routes raise no page errors", result.pageErrors.join(" | "));
  check(await page.getByRole("link", { name: "Privacy" }).first().isVisible() && await page.getByRole("link", { name: "Terms" }).first().isVisible(), "404 retains legal links");
  check(Boolean(await page.locator('meta[name="description"]').getAttribute("content")), "404 retains metadata");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  check(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), "404 reflows at 390 pixels and 200 percent text");
  const recovery = await page.getByRole("link", { name: "Go to Class Capacity Truth" }).boundingBox();
  check(Boolean(recovery && recovery.height >= 44), "404 recovery target is at least 44 pixels high", JSON.stringify(recovery));
  await desktopContext.close();
  result.verdict = "PASS";
} finally {
  await browser.close();
  await writeFile(".factory/review-3-evidence/live-surfaces.json", `${JSON.stringify(result, null, 2)}\n`);
}
