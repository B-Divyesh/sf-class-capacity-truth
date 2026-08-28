export const foundationTitle = "Class Capacity Truth — Show the right seat count";

export const plannedRoutes = [
  { path: "/", title: foundationTitle },
  { path: "/demo", title: "Demo — Class Capacity Truth" },
  { path: "/book/:publicClassId", title: "Book a class — Class Capacity Truth" },
  { path: "/privacy", title: "Privacy — Class Capacity Truth" },
  { path: "/terms", title: "Terms — Class Capacity Truth" },
  { path: "/404", title: "Page not found — Class Capacity Truth" }
] as const;

export function titleForPath(pathname: string): string {
  if (pathname.startsWith("/demo")) {
    return "Demo — Class Capacity Truth";
  }

  if (pathname.startsWith("/book/")) {
    return "Book a class — Class Capacity Truth";
  }

  return plannedRoutes.find((route) => route.path === pathname)?.title ?? foundationTitle;
}
