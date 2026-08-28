import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test.beforeEach(async ({ context }, testInfo) => {
  const seed = [...testInfo.title].reduce((total, character) => total + character.charCodeAt(0), 0);
  await context.setExtraHTTPHeaders({ "x-forwarded-for": `198.51.100.${10 + (seed % 200)}` });
});

test("@claim:sample-booking-updates-seats books one seat and updates the count", async ({ page }) => {
  await page.goto("/demo?demo=1");
  const openClass = page.getByRole("article").filter({ hasText: "Level check: upper primary" });
  await expect(openClass.getByText("2 seats open", { exact: false })).toBeVisible();
  await openClass.getByRole("link", { name: "Book this sample class" }).click();
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await expect(page.getByRole("heading", { name: "Your sample seat is booked" })).toBeVisible();
  await expect(page.getByText("1 seat is now open in this class.")).toBeVisible();
  await expect(page.getByRole("img", { name: "7 confirmed, 1 open" })).toBeVisible();
});

test("@claim:full-class-blocks-booking blocks a full class", async ({ page }) => {
  await page.goto("/demo?demo=1");
  const fullClass = page.getByRole("article").filter({ hasText: "Friday conversation group" });
  await expect(fullClass.getByText("Full · 0 seats open")).toBeVisible();
  await fullClass.getByRole("link", { name: "View the full class" }).press("Enter");
  await expect(page.getByRole("heading", { name: "This class is full" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Book one sample seat" })).toHaveCount(0);

  const classId = new URL(page.url()).pathname.split("/").pop()!;
  const response = await page.request.post(`/api/demo/classes/${classId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID() },
    data: { guardianName: "Alex Morgan", guardianEmail: "alex@example.org" }
  });
  expect(response.status()).toBe(409);
  expect((await response.json()).code).toBe("class_full");
});

test("@claim:cutoff-blocks-booking blocks a class after its cutoff", async ({ page }) => {
  await page.goto("/demo?demo=1");
  const closedClass = page.getByRole("article").filter({ hasText: "Saturday assessment" });
  await expect(closedClass.getByText("Booking closed", { exact: false })).toBeVisible();
  await closedClass.getByRole("link", { name: "View the closed class" }).click();
  await expect(page.getByRole("heading", { name: "Booking has closed" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Book one sample seat" })).toHaveCount(0);

  const classId = new URL(page.url()).pathname.split("/").pop()!;
  const response = await page.request.post(`/api/demo/classes/${classId}/book`, {
    headers: { "Idempotency-Key": crypto.randomUUID() },
    data: { guardianName: "Alex Morgan", guardianEmail: "alex@example.org" }
  });
  expect(response.status()).toBe(409);
  expect((await response.json()).code).toBe("booking_closed");
});

test("@claim:demo-reset-isolated keeps browser demos separate and resets changes", async ({ browser, baseURL }) => {
  const firstContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "203.0.113.31" } });
  const secondContext = await browser.newContext({ extraHTTPHeaders: { "x-forwarded-for": "203.0.113.32" } });
  const first = await firstContext.newPage();
  const second = await secondContext.newPage();
  const outgoing: string[] = [];
  first.on("request", (request) => outgoing.push(request.url()));

  await first.goto("/demo?demo=1");
  await first.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await first.getByRole("button", { name: "Book one sample seat" }).click();
  await expect(first.getByText("1 seat is now open in this class.")).toBeVisible();
  await first.getByRole("button", { name: "Reset demo" }).click();
  await expect(first).toHaveURL(/\/demo\?demo=1$/);
  await expect(first.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false })).toBeVisible();

  await second.goto("/demo?demo=1");
  await expect(second.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false })).toBeVisible();
  const origin = new URL(baseURL!).origin;
  expect(outgoing.every((url) => new URL(url).origin === origin)).toBe(true);
  await firstContext.close();
  await secondContext.close();
});

test("keyboard booking and route focus work", async ({ page }) => {
  await page.goto("/demo?demo=1");
  const link = page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" });
  await link.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { level: 1 })).toBeFocused();
  await expect(page).toHaveTitle("Book a class — Class Capacity Truth");
  await page.getByLabel("Guardian name").focus();
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { name: "Your sample seat is booked" })).toBeVisible();
  await page.getByRole("link", { name: "Privacy" }).first().click();
  await expect(page.getByRole("heading", { level: 1, name: "Privacy in the sample" })).toBeFocused();
  await expect(page).toHaveTitle("Privacy — Class Capacity Truth");
});

test("resetting from a booking returns to fresh sample classes", async ({ page }) => {
  await page.goto("/demo?demo=1");
  await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await page.getByRole("button", { name: "Book one sample seat" }).click();
  await page.getByRole("button", { name: "Reset demo" }).click();
  await expect(page).toHaveURL(/\/demo\?demo=1$/);
  await expect(page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByText("2 seats open", { exact: false })).toBeVisible();
});

test("axe finds no serious issues on a booking route", async ({ page }) => {
  await page.goto("/demo?demo=1");
  await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
});

test("dark treatment has no serious contrast issues", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/");
  let results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);

  await page.goto("/demo?demo=1");
  await page.getByRole("article").filter({ hasText: "Level check: upper primary" }).getByRole("link", { name: "Book this sample class" }).click();
  results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
});

test("the demo remains usable at 390px and with reduced motion", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/demo?demo=1");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.getByRole("button", { name: "Reset demo" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  expect(await page.locator(".loading-state > span").count()).toBe(0);
});

test("release regression: hashed assets are immutable and unknown paths are HTTP 404", async ({ page }) => {
  const assetHeaders: string[] = [];
  page.on("response", (response) => { if (response.url().includes("/assets/")) assetHeaders.push(response.headers()["cache-control"] ?? ""); });
  await page.goto("/");
  expect(assetHeaders).toContain("public, max-age=31536000, immutable");
  const missing = await page.goto("/missing-page");
  expect(missing?.status()).toBe(404);
  await expect(page.getByRole("heading", { level: 1 })).toHaveCount(1);
});

test("school workspace stays usable at 390px", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/app");
  await expect(page.getByRole("heading", { name: "Create your school workspace" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Create school workspace" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
});

test("@claim:school-capacity-flow creates, publishes, reconciles, books, waits, and converts a released seat", async ({ page }) => {
  await page.goto("/app");
  await page.getByLabel("School name").fill("Harbour Languages");
  await page.getByRole("button", { name: "Create school workspace" }).click();
  await expect(page.getByRole("heading", { name: "Manage class capacity" })).toBeVisible();
  await page.getByLabel("Class name").fill("Saturday level check");
  await page.getByLabel("Starts at").fill("2030-06-10T10:00");
  await page.getByLabel("Booking cutoff").fill("2030-06-09T10:00");
  await page.getByLabel("Capacity").fill("2");
  await page.getByRole("button", { name: "Create class" }).click();
  const classCard = page.getByRole("article").filter({ hasText: "Saturday level check" });
  await classCard.getByRole("button", { name: "Publish parent link" }).click();
  const href = await classCard.getByRole("link", { name: "Open booking page" }).getAttribute("href");
  expect(href).toMatch(/^\/book\/class_/);
  await page.getByLabel("Calendar confirmed bookings for Saturday level check").fill("1");
  await page.getByLabel("Calendar confirmed bookings for Saturday level check").blur();
  await expect(page.getByRole("heading", { name: "Manage class capacity" })).toBeVisible();
  await page.goto(href!);
  await page.getByLabel("Guardian name").fill("Alex Morgan");
  await page.getByLabel("Email address").fill("alex@example.org");
  await page.getByRole("button", { name: "Book this seat" }).click();
  await expect(page.getByText("Your place is confirmed.")).toBeVisible();
  const publicId = href!.split("/").pop()!;
  await page.evaluate(async ({ publicId }) => {
    await fetch(`/api/classes/${publicId}/waitlist`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ guardianName: "Waiting Parent", guardianEmail: "waiting@example.org", consent: true }) });
  }, { publicId });
  await page.goto("/app");
  await classCard.getByRole("button", { name: "Release one confirmed seat" }).click();
  await expect(page.getByText(/Released seat offer created:/)).toBeVisible();
  const text = await page.getByText(/Released seat offer created:/).textContent();
  const token = text!.match(/\/offer\/(offer_[a-z0-9]+)/)?.[1];
  expect(token).toBeTruthy();
  await page.goto(`/offer/${token}`);
  await page.getByRole("button", { name: "Accept this seat" }).click();
  await expect(page.getByText(/Your released seat is confirmed/)).toBeVisible();
});

for (const route of ["/", "/demo?demo=1", "/privacy", "/terms", "/missing-page"]) {
  test(`axe finds no serious issues on ${route}`, async ({ page }) => {
    const browserErrors: string[] = [];
    page.on("console", (message) => { if (message.type() === "error") browserErrors.push(`${message.text()} ${message.location().url}`); });
    const response = await page.goto(route);
    if (route === "/missing-page") expect(response?.status()).toBe(404);
    await expect(page.getByRole("heading", { level: 1 })).toHaveCount(1);
    if (route.startsWith("/demo")) await expect(page.getByRole("article")).toHaveCount(3);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
    expect(browserErrors.filter((error) => !(route === "/missing-page" && error.includes("404")))).toEqual([]);
  });
}
