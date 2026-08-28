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
});
