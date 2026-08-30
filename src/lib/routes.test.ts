import { describe, expect, it } from "vitest";
import { routeForPath, titleForPath } from "./routes";

describe("route titles", () => {
  it("keeps the plain-words home title", () => {
    expect(titleForPath("/")).toBe("Class Capacity Truth — Show the right seat count");
  });

  it("assigns named titles to planned public routes", () => {
    expect(titleForPath("/demo")).toBe("Demo — Class Capacity Truth");
    expect(titleForPath("/book/sample-class")).toBe("Book a class — Class Capacity Truth");
    expect(titleForPath("/privacy")).toBe("Privacy — Class Capacity Truth");
    expect(routeForPath("/missing").kind).toBe("notFound");
  });

  it("recognises every shipped workspace deep link", () => {
    const expected = [
      ["/app/classes/example", "Class capacity — Class Capacity Truth", "classDetail"],
      ["/app/reconciliation", "Calendar checks — Class Capacity Truth", "reconciliation"],
      ["/app/waitlist", "Waitlist offers — Class Capacity Truth", "waitlist"],
      ["/app/settings", "Settings — Class Capacity Truth", "settings"],
      ["/app/settings/billing", "Billing — Class Capacity Truth", "billing"],
      ["/app/settings/data", "School data — Class Capacity Truth", "data"],
      ["/app/operations", "Operations — Class Capacity Truth", "operations"]
    ] as const;
    for (const [path, title, section] of expected) {
      const route = routeForPath(path);
      expect(route.kind).toBe("workspace");
      expect(route.title).toBe(title);
      expect(route.workspaceSection).toBe(section);
    }
  });
});
