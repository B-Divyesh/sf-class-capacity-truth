import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("foundation app", () => {
  it("has one clear page heading and a skip link", () => {
    render(<App />);

    expect(
      screen.getByRole("heading", { level: 1, name: "The capacity product is planned." })
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Skip to main content" })).toHaveAttribute(
      "href",
      "#main"
    );
  });
});
