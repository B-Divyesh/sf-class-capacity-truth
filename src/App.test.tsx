import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App, zonedDateTimeToEpoch } from "./App";
import { bearerFromAuthenticationResult } from "./lib/auth";

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

  it("exposes a labelled mobile navigation disclosure", () => {
    render(<App />);
    const menu = screen.getByRole("button", { name: "Open main menu" });

    expect(menu).toHaveAttribute("aria-controls", "main-navigation");
    expect(menu).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(menu);
    expect(screen.getByRole("button", { name: "Close main menu" })).toHaveAttribute(
      "aria-expanded",
      "true"
    );
  });
});

describe("school wall-time conversion", () => {
  it("uses the selected school zone instead of the browser zone", () => {
    expect(zonedDateTimeToEpoch("2030-06-10T10:00", "Europe/London")).toBe(
      Date.UTC(2030, 5, 10, 9, 0) / 1000
    );
    expect(zonedDateTimeToEpoch("2030-06-10T10:00", "America/New_York")).toBe(
      Date.UTC(2030, 5, 10, 14, 0) / 1000
    );
  });
});

describe("Entra bearer selection", () => {
  it("uses the ID token when an OIDC-only response has no access token", () => {
    expect(bearerFromAuthenticationResult({ accessToken: "", idToken: "signed-id-token" })).toBe("signed-id-token");
  });
});
