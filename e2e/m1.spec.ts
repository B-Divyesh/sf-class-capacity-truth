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

for (const route of ["/", "/demo?demo=1", "/privacy", "/terms", "/missing-page"]) {
  test(`axe finds no serious issues on ${route}`, async ({ page }) => {
    const browserErrors: string[] = [];
    page.on("console", (message) => { if (message.type() === "error") browserErrors.push(`${message.text()} ${message.location().url}`); });
    await page.goto(route);
    await expect(page.getByRole("heading", { level: 1 })).toHaveCount(1);
    if (route.startsWith("/demo")) await expect(page.getByRole("article")).toHaveCount(3);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
    expect(browserErrors).toEqual([]);
  });
}
