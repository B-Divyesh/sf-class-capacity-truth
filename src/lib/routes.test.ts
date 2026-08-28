import { describe, expect, it } from "vitest";
import { foundationTitle, titleForPath } from "./routes";

describe("route titles", () => {
  it("keeps the plain-words home title", () => {
    expect(foundationTitle).toBe("Class Capacity Truth — Show the right seat count");
  });

  it("assigns named titles to planned public routes", () => {
    expect(titleForPath("/demo")).toBe("Demo — Class Capacity Truth");
    expect(titleForPath("/book/sample-class")).toBe("Book a class — Class Capacity Truth");
    expect(titleForPath("/privacy")).toBe("Privacy — Class Capacity Truth");
  });
});
