import fs from "node:fs/promises";
import { chromium } from "@playwright/test";

const productOrigin = "https://class-capacity-truth.sociobot.in";
const endpoint = "https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout";
const report = { requestMethod: null, endpointResponseStatus: null, failedHost: null, failure: null, alert: null, finalHost: null, finalPath: null, consoleErrors: [] };
const browser = await chromium.launch();
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on("request", (request) => {
    if (request.url() === endpoint) report.requestMethod = request.method();
  });
  page.on("response", (response) => {
    if (response.url() === endpoint) report.endpointResponseStatus = response.status();
  });
  page.on("requestfailed", (request) => {
    const url = new URL(request.url());
    if (request.url() === endpoint || url.hostname === "checkout.dodopayments.com") {
      report.failedHost = url.hostname;
      report.failure = request.failure()?.errorText ?? "unknown";
    }
  });
  page.on("console", (message) => {
    if (message.type() === "error") report.consoleErrors.push(message.text().replace(/https:\/\/checkout\.dodopayments\.com\/\S+/g, "https://checkout.dodopayments.com/<redacted>"));
  });
  await page.goto(`${productOrigin}/app/settings/billing`, { waitUntil: "networkidle" });
  await page.getByRole("button", { name: /Start the \$99-per-school monthly plan/ }).click();
  const alert = page.getByRole("alert");
  await alert.waitFor({ state: "visible", timeout: 15_000 });
  report.alert = await alert.textContent();
  const final = new URL(page.url());
  report.finalHost = final.hostname;
  report.finalPath = final.pathname.startsWith("/session/") ? "/session/<redacted>" : final.pathname;
  await page.screenshot({ path: ".factory/review-5-evidence/live-checkout.png", fullPage: true });
  await fs.writeFile(".factory/review-5-evidence/live-checkout.json", `${JSON.stringify(report, null, 2)}\n`);
  console.log(JSON.stringify(report, null, 2));
} finally {
  await browser.close();
}
