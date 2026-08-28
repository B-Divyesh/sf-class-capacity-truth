export type RouteKind = "home" | "demo" | "booking" | "workspace" | "offer" | "privacy" | "terms" | "notFound";

export interface RouteInfo {
  kind: RouteKind;
  title: string;
  description: string;
  publicClassId?: string;
  offerToken?: string;
}

const routes: Record<Exclude<RouteKind, "booking">, Omit<RouteInfo, "kind">> = {
  home: {
    title: "Class Capacity Truth — Show the right seat count",
    description: "Try a sample class booking and see the available seat count change."
  },
  demo: {
    title: "Demo — Class Capacity Truth",
    description: "Book, block, and reset sample class seats in an isolated demo."
  },
  workspace: {
    title: "School workspace — Class Capacity Truth",
    description: "Create, publish, and reconcile trustworthy class capacity."
  },
  offer: {
    title: "Claim a seat — Class Capacity Truth",
    description: "Accept a released class seat offer."
  },
  privacy: {
    title: "Privacy — Class Capacity Truth",
    description: "How Class Capacity Truth handles sample booking information."
  },
  terms: {
    title: "Terms — Class Capacity Truth",
    description: "Terms for the Class Capacity Truth sample and future school plan."
  },
  notFound: {
    title: "Page not found — Class Capacity Truth",
    description: "This Class Capacity Truth page could not be found."
  }
};

export function routeForPath(pathname: string): RouteInfo {
  if (pathname === "/") return { kind: "home", ...routes.home };
  if (pathname === "/demo") return { kind: "demo", ...routes.demo };
  if (pathname === "/app") return { kind: "workspace", ...routes.workspace };
  if (pathname === "/privacy") return { kind: "privacy", ...routes.privacy };
  if (pathname === "/terms") return { kind: "terms", ...routes.terms };
  if (pathname === "/404") return { kind: "notFound", ...routes.notFound };
  const match = pathname.match(/^\/book\/([a-zA-Z0-9_-]+)$/);
  if (match) {
    return {
      kind: "booking",
      title: "Book a class — Class Capacity Truth",
      description: "See a sample class seat count and book one available seat.",
      publicClassId: match[1]
    };
  }
  const offer = pathname.match(/^\/offer\/([a-zA-Z0-9_-]+)$/);
  if (offer) return { kind: "offer", ...routes.offer, offerToken: offer[1] };
  return { kind: "notFound", ...routes.notFound };
}

export function titleForPath(pathname: string): string {
  return routeForPath(pathname).title;
}
