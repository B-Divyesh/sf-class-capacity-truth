import fs from "node:fs/promises";
import { chromium, request } from "@playwright/test";

const baseURL = "https://class-capacity-truth.sociobot.in";
const routes = ["/", "/demo?demo=1", "/app", "/app/reconciliation", "/app/waitlist", "/app/settings", "/app/settings/billing", "/app/settings/data", "/app/operations", "/privacy", "/terms", "/not-a-real-page"];
const browser = await chromium.launch();
const context = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "198.51.100.192" } });
const page = await context.newPage();
const hrefs = new Set();
for (const route of routes) {
  await page.goto(`${baseURL}${route}`, { waitUntil: "networkidle" });
  if (route.startsWith("/demo")) await page.getByRole("article").first().waitFor();
  for (const href of await page.locator("a").evaluateAll((items) => items.map((item) => item.href).filter(Boolean))) hrefs.add(href);
}
const client = await request.newContext();
const results = [];
for (const href of [...hrefs].sort()) {
  if (href.startsWith("mailto:")) {
    results.push({ href, status: "mailto" });
    continue;
  }
  const response = await client.get(href, { maxRedirects: 5 });
  const expected404Anchor = new URL(href).pathname === "/not-a-real-page" && new URL(href).hash === "#main" && response.status() === 404;
  results.push({ href, status: expected404Anchor ? "expected-404-anchor" : response.status() });
}
await client.dispose();
await context.close();
await browser.close();
await fs.writeFile(".factory/review-4-evidence/link-crawl.json", `${JSON.stringify(results, null, 2)}\n`);
const failures = results.filter((item) => typeof item.status === "number" && (item.status < 200 || item.status >= 400));
console.log(JSON.stringify({ checked: results.length, failures }));
if (failures.length) process.exit(1);
