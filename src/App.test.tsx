import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("public app", () => {
  it("has one clear page heading and a skip link", () => {
    render(<App />);

    expect(
      screen.getByRole("heading", { level: 1, name: "Show the right number of class seats" })
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Skip to main content" })).toHaveAttribute(
      "href",
      "#main"
    );
  });

  it("offers the sample in one click", () => {
    render(<App />);
    expect(screen.getByRole("link", { name: "Try it with sample data" })).toHaveAttribute(
      "href",
      "/demo?demo=1"
    );
  });
});
