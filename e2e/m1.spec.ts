import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test.beforeEach(async ({ context }, testInfo) => {
  const seed = [...testInfo.title].reduce((total, character) => total + character.charCodeAt(0), 0) + testInfo.retry * 211 + testInfo.workerIndex * 17;
  await context.setExtraHTTPHeaders({ "x-forwarded-for": `198.51.100.${10 + (seed % 200)}` });
  await context.addInitScript((token) => sessionStorage.setItem("cct:test-access-token", token), `test-owner-${seed}`);
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
  await page.goto("/");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to main content" })).toBeFocused();
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
  await expect(page.getByRole("heading", { level: 1, name: "Privacy for bookings and the demo" })).toBeFocused();
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

test("release regression: standalone 404 reflows at 390px with 200 percent text and a 44px recovery link", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const response = await page.goto("/missing-page");
  expect(response?.status()).toBe(404);
  await page.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
  await expect(page.getByRole("heading", { level: 1, name: "This page was not found." })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  const recovery = page.getByRole("link", { name: "Go to Class Capacity Truth" });
  await expect(recovery).toBeVisible();
  const box = await recovery.boundingBox();
  expect(box?.height).toBeGreaterThanOrEqual(44);
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

test("all routes reflow at 390px and 200 percent text size with 44px targets", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const route of ["/", "/demo?demo=1", "/app", "/privacy", "/terms"]) {
    await page.goto(route);
    await page.evaluate(() => { document.documentElement.style.fontSize = "200%"; });
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    if (route.startsWith("/demo")) await expect(page.getByRole("article")).toHaveCount(3);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth), route).toBe(true);
  }
  await page.evaluate(() => { document.documentElement.style.fontSize = "100%"; });
  for (const route of ["/", "/privacy", "/terms"]) {
    await page.goto(route);
    const sizes = await page.locator("main a, main button, footer a").evaluateAll((items) => items.map((item) => ({
      label: item.textContent?.trim(), width: item.getBoundingClientRect().width, height: item.getBoundingClientRect().height
    })));
    expect(sizes.filter((size) => size.width < 44 || size.height < 44), route).toEqual([]);
  }
});

test("@claim:school-capacity-flow @claim:released-seat-delivery creates, copies, reloads, and converts", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"], { origin: "http://127.0.0.1:4173" });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/app");
  await page.getByLabel("School name").fill("Harbour Languages");
  await page.getByRole("button", { name: "Create school workspace" }).click();
  await expect(page.getByRole("heading", { name: "Manage class capacity" })).toBeVisible();
  await expect(page.getByText("This deployment does not send email. Cancelling creates a one-click offer below. Copy it and send it through your school's approved channel.")).toBeVisible();
  const runtime = await page.request.get("/api/runtime");
  expect(await runtime.json()).toEqual({ emailDelivery: "not_configured" });
  await page.getByLabel("Class name").fill("Saturday level check");
  await page.getByLabel("Starts at").fill("2030-06-10T10:00");
  await page.getByLabel("Booking cutoff").fill("2030-06-09T10:00");
  await page.getByLabel("Capacity").fill("1");
  await page.getByRole("button", { name: "Create class" }).click();
  const classCard = page.getByRole("article").filter({ hasText: "Saturday level check" });
  await classCard.getByRole("button", { name: "Publish parent link" }).click();
  const href = await classCard.getByRole("link", { name: "Open booking page" }).getAttribute("href");
  expect(href).toMatch(/^\/book\/class_/);
  await page.getByLabel("Calendar label").fill("School bookings calendar");
  await page.getByLabel("iCalendar feed URL").fill("https://fixture.invalid/school.ics");
  await page.getByRole("button", { name: "Connect and check calendar" }).click();
  await expect(page.getByText(/Calendar connected and checked/)).toBeVisible();
  await page.goto(href!);
  await page.getByLabel("Guardian name").fill("Alex Morgan");
  await page.getByLabel("Email address").fill("alex@example.org");
  await page.getByRole("button", { name: "Book this seat" }).click();
  await expect(page.getByText("Your place is confirmed.")).toBeVisible();
  await page.reload();
  await page.getByLabel("Guardian name").fill("Waiting Parent");
  await page.getByLabel("Email address").fill("waiting@example.org");
  await page.getByRole("button", { name: "Join waitlist" }).click();
  await expect(page.getByText(/You are on the waitlist/)).toBeVisible();
  await page.goto("/app");
  await classCard.getByRole("button", { name: "Choose a booking to cancel" }).click();
  await expect(classCard.getByText("Alex Morgan", { exact: true })).toBeVisible();
  page.once("dialog", (dialog) => dialog.accept());
  const [cancelResponse] = await Promise.all([
    page.waitForResponse((response) => response.url().includes("/bookings/") && response.url().endsWith("/cancel")),
    classCard.getByRole("button", { name: "Cancel Alex Morgan booking" }).click()
  ]);
  const token = (await cancelResponse.json()).offerToken as string;
  await expect(page.getByText(/Copy the one-click offer below/)).toBeVisible();
  expect(token).toBeTruthy();
  const receipt = page.getByRole("article").filter({ hasText: "Ready to share — no email was sent." });
  const offerUrl = await receipt.getByLabel("One-click offer URL").inputValue();
  expect(offerUrl).toBe(`https://class-capacity-truth.sociobot.in/offer/${token}`);
  await receipt.getByRole("button", { name: "Copy offer" }).focus();
  await page.keyboard.press("Enter");
  await expect(receipt.getByText("Offer copied.")).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(offerUrl);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  const receiptAxe = await new AxeBuilder({ page }).analyze();
  expect(receiptAxe.violations.filter((item) => ["serious", "critical"].includes(item.impact ?? ""))).toEqual([]);
  await page.reload();
  await expect(page.getByLabel("One-click offer URL")).toHaveValue(offerUrl);
  await page.goto(new URL(offerUrl).pathname);
  await page.getByRole("button", { name: "Accept this seat" }).click();
  await expect(page.getByText(/Your released seat is confirmed/)).toBeVisible();
});

test("release regression: school wall time does not drift with the browser zone", async ({ browser }) => {
  const context = await browser.newContext({ timezoneId: "America/New_York", extraHTTPHeaders: { "x-forwarded-for": "203.0.113.90" } });
  await context.addInitScript(() => sessionStorage.setItem("cct:test-access-token", "test-owner-timezone"));
  const page = await context.newPage();
  await page.goto("/app");
  await page.getByLabel("School name").fill("Timezone School");
  await page.getByRole("button", { name: "Create school workspace" }).click();
  await page.getByLabel("Class name").fill("London morning class");
  await page.getByLabel("School time zone").selectOption("Europe/London");
  await page.getByLabel("Starts at").fill("2030-06-10T10:00");
  await page.getByLabel("Booking cutoff").fill("2030-06-09T10:00");
  await page.getByLabel("Capacity").fill("2");
  await page.getByRole("button", { name: "Create class" }).click();
  await expect(page.getByRole("article").filter({ hasText: "London morning class" })).toContainText("10:00");
  await context.close();
});

test("@claim:school-plan-price shows the exact price and opens hosted Sociobot checkout", async ({ page }) => {
  let checkoutMethod = "";
  await page.route("https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout", async (route) => {
    checkoutMethod = route.request().method();
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ checkout_url: "https://checkout.dodopayments.com/session/test_class_capacity_truth" }) });
  });
  await page.route("https://checkout.dodopayments.com/**", (route) => route.fulfill({ status: 200, contentType: "text/html", body: "<!doctype html><title>Hosted checkout</title>" }));
  await page.goto("/");
  await expect(page.getByText("The school plan costs $99 each month.")).toBeVisible();
  await page.getByRole("button", { name: /Open the \$99 monthly Sociobot checkout/ }).click();
  await page.waitForURL("https://checkout.dodopayments.com/session/test_class_capacity_truth");
  expect(checkoutMethod).toBe("POST");
});

test("@claim:data-export-delete exports and deletes the workspace", async ({ page }) => {
  await page.goto("/app");
  await page.getByLabel("School name").fill("Data Rights School");
  await page.getByRole("button", { name: "Create school workspace" }).click();
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export school data" }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe("class-capacity-truth-export.json");
  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Delete school workspace" }).click();
  await expect(page.getByRole("heading", { name: "Create your school workspace" })).toBeVisible();
});

test("calendar connection UI checks the feed without changing confirmed seats", async ({ page }) => {
  await page.goto("/app");
  await page.getByLabel("School name").fill("Non-mutating Calendar School");
  await page.getByRole("button", { name: "Create school workspace" }).click();
  await page.getByLabel("Class name").fill("Saturday level check");
  await page.getByLabel("Starts at").fill("2030-06-10T10:00");
  await page.getByLabel("Booking cutoff").fill("2030-06-09T10:00");
  await page.getByLabel("Capacity").fill("3");
  await page.getByRole("button", { name: "Create class" }).click();
  const classCard = page.getByRole("article").filter({ hasText: "Saturday level check" });
  await expect(classCard.getByRole("img", { name: "0 confirmed, 3 open" })).toBeVisible();
  await classCard.getByRole("button", { name: "Publish parent link" }).click();
  await page.getByLabel("Calendar label").fill("School bookings calendar");
  await page.getByLabel("iCalendar feed URL").fill("https://fixture.invalid/school.ics");
  await page.getByRole("button", { name: "Connect and check calendar" }).click();
  await expect(page.getByText(/Calendar connected and checked/)).toBeVisible();
  await expect(classCard).toContainText("Attention: calendar says 1, local ledger says 0.");
  await expect(classCard.getByRole("img", { name: "0 confirmed, 3 open" })).toBeVisible();
});

test("@claim:no-third-party-tracking observed product flows stay same-origin", async ({ page, baseURL }) => {
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));
  for (const route of ["/", "/demo?demo=1", "/privacy", "/app"]) {
    await page.goto(route);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  }
  const origin = new URL(baseURL!).origin;
  expect(requests.every((url) => new URL(url).origin === origin)).toBe(true);
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
