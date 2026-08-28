import { expect, test } from "@playwright/test";

test("the foundation shell loads without browser errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(message.text());
    }
  });

  await page.goto("/");
  await expect(page).toHaveTitle("Class Capacity Truth — Show the right seat count");
  await expect(page.getByRole("heading", { level: 1 })).toHaveCount(1);
  await expect(page.locator("main")).toBeVisible();
  expect(errors).toEqual([]);
});
