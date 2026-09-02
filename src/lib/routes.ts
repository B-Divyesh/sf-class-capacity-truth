export type RouteKind = "home" | "demo" | "booking" | "workspace" | "authCallback" | "offer" | "privacy" | "terms" | "notFound";
export type WorkspaceSection = "classes" | "classDetail" | "reconciliation" | "waitlist" | "settings" | "billing" | "data" | "operations";

export interface RouteInfo {
  kind: RouteKind;
  title: string;
  description: string;
  publicClassId?: string;
  offerToken?: string;
  workspaceSection?: WorkspaceSection;
  workspaceClassId?: string;
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
    title: "Classes — Class Capacity Truth",
    description: "Create, publish, and inspect trustworthy class capacity."
  },
  authCallback: {
    title: "Finish sign in — Class Capacity Truth",
    description: "Finish secure staff sign in for Class Capacity Truth."
  },
  offer: {
    title: "Claim a seat — Class Capacity Truth",
    description: "Accept a released class seat offer."
  },
  privacy: {
    title: "Privacy — Class Capacity Truth",
    description: "How Class Capacity Truth handles demo and school booking information."
  },
  terms: {
    title: "Terms — Class Capacity Truth",
    description: "Terms for the demo and the $99-per-school monthly plan."
  },
  notFound: {
    title: "Page not found — Class Capacity Truth",
    description: "This Class Capacity Truth page could not be found."
  }
};

export function routeForPath(pathname: string): RouteInfo {
  if (pathname === "/") return { kind: "home", ...routes.home };
  if (pathname === "/demo") return { kind: "demo", ...routes.demo };
  if (pathname === "/app") return { kind: "workspace", ...routes.workspace, workspaceSection: "classes" };
  const workspaceClass = pathname.match(/^\/app\/classes\/([a-zA-Z0-9_-]+)$/);
  if (workspaceClass) return {
    kind: "workspace",
    title: "Class capacity — Class Capacity Truth",
    description: "Inspect one class capacity, bookings, and its public booking link.",
    workspaceSection: "classDetail",
    workspaceClassId: workspaceClass[1]
  };
  const workspaceRoutes: Record<string, Omit<RouteInfo, "kind">> = {
    "/app/reconciliation": {
      title: "Calendar checks — Class Capacity Truth",
      description: "Connect a calendar and review capacity differences.",
      workspaceSection: "reconciliation"
    },
    "/app/waitlist": {
      title: "Waitlist offers — Class Capacity Truth",
      description: "Review and share released-seat offers.",
      workspaceSection: "waitlist"
    },
    "/app/settings": {
      title: "Settings — Class Capacity Truth",
      description: "Open school billing, data, and sign-in settings.",
      workspaceSection: "settings"
    },
    "/app/settings/billing": {
      title: "Billing — Class Capacity Truth",
      description: "Verify a school plan purchase or open the $99-per-school monthly checkout.",
      workspaceSection: "billing"
    },
    "/app/settings/data": {
      title: "School data — Class Capacity Truth",
      description: "Export or delete a school workspace.",
      workspaceSection: "data"
    },
    "/app/operations": {
      title: "Operations — Class Capacity Truth",
      description: "Review capacity discrepancies, calendar lag, and service health.",
      workspaceSection: "operations"
    }
  };
  if (workspaceRoutes[pathname]) return { kind: "workspace", ...workspaceRoutes[pathname] };
  if (pathname === "/auth/callback") return { kind: "authCallback", ...routes.authCallback };
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
