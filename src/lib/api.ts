export type Availability = "available" | "full" | "cutoff";

export interface ClassSession {
  publicId: string;
  name: string;
  startsAt: number;
  bookingCutoff: number;
  timezone: string;
  capacity: number;
  confirmed: number;
  openSeats: number;
  availability: Availability;
}

export interface DemoData {
  schoolName: string;
  expiresAt: number;
  classes: ClassSession[];
}

export interface RealClass extends ClassSession {
  id: string;
  published: boolean;
  calendarConfirmed: number | null;
  reconciliationStatus: "matched" | "attention" | null;
}

export interface Workspace { id: string; schoolName: string }
const workspaceHeader = (): Record<string, string> => {
  const key = localStorage.getItem("cct:workspace-key");
  return key ? { "X-Workspace-Key": key } : {};
};
export function workspaceKey() { return localStorage.getItem("cct:workspace-key"); }
export async function createWorkspace(schoolName: string) {
  const result = await responseJson<{ workspace: Workspace; accessKey: string }>(await fetch("/api/workspaces", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ schoolName }) }));
  localStorage.setItem("cct:workspace-key", result.accessKey);
  return result.workspace;
}
export async function loadWorkspace() { return responseJson<Workspace>(await fetch("/api/workspaces", { headers: workspaceHeader() })); }
export async function listClasses() { return responseJson<RealClass[]>(await fetch("/api/workspaces/classes", { headers: workspaceHeader() })); }
export async function createClass(input: { name: string; startsAt: number; bookingCutoff: number; timezone: string; capacity: number }) {
  return responseJson<RealClass>(await fetch("/api/workspaces/classes", { method: "POST", headers: { "Content-Type": "application/json", ...workspaceHeader() }, body: JSON.stringify(input) }));
}
export async function publishClass(id: string) { return responseJson<RealClass>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/publish`, { method: "POST", headers: workspaceHeader() })); }
export async function connectCalendar(label: string) { return responseJson<{ label: string; provider: string; enabled: boolean }>(await fetch("/api/workspaces/calendar", { method: "PUT", headers: { "Content-Type": "application/json", ...workspaceHeader() }, body: JSON.stringify({ label }) })); }
export async function reconcileClass(id: string, calendarConfirmed: number) { return responseJson<RealClass>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/reconcile`, { method: "POST", headers: { "Content-Type": "application/json", ...workspaceHeader() }, body: JSON.stringify({ calendarConfirmed }) })); }
export async function releaseSeat(id: string) { return responseJson<{ offerToken: string | null }>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/release-seat`, { method: "POST", headers: workspaceHeader() })); }
export async function loadPublicClass(publicId: string) { return responseJson<RealClass>(await fetch(`/api/classes/${encodeURIComponent(publicId)}`)); }
export async function bookRealClass(publicId: string, guardianName: string, guardianEmail: string, idempotencyKey: string) { return responseJson<RealClass>(await fetch(`/api/classes/${encodeURIComponent(publicId)}/book`, { method: "POST", headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey }, body: JSON.stringify({ guardianName, guardianEmail }) })); }
export async function joinWaitlist(publicId: string, guardianName: string, guardianEmail: string) { return responseJson<void>(await fetch(`/api/classes/${encodeURIComponent(publicId)}/waitlist`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ guardianName, guardianEmail, consent: true }) })); }
export async function loadOffer(token: string) { return responseJson<{ offerToken: string; class: RealClass; expiresAt: number }>(await fetch(`/api/offers/${encodeURIComponent(token)}`)); }
export async function acceptOffer(token: string) { return responseJson<RealClass>(await fetch(`/api/offers/${encodeURIComponent(token)}/accept`, { method: "POST" })); }

interface ApiErrorBody { code?: string; message?: string }

export class ApiError extends Error {
  constructor(message: string, public readonly status: number, public readonly code?: string) {
    super(message);
  }
}

async function responseJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as ApiErrorBody;
    throw new ApiError(body.message ?? "The request did not finish. Try again.", response.status, body.code);
  }
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}

export async function loadDemo(signal?: AbortSignal): Promise<DemoData> {
  return responseJson<DemoData>(await fetch("/api/demo/session", { credentials: "same-origin", signal }));
}

export async function resetDemo(): Promise<DemoData> {
  return responseJson<DemoData>(await fetch("/api/demo/reset", { method: "POST", credentials: "same-origin" }));
}

export async function leaveDemo(): Promise<void> {
  return responseJson<void>(await fetch("/api/demo/leave", { method: "POST", credentials: "same-origin" }));
}

export async function bookSeat(publicClassId: string, guardianName: string, guardianEmail: string, idempotencyKey: string) {
  return responseJson<{ bookingId: string; class: ClassSession; repeated: boolean }>(
    await fetch(`/api/demo/classes/${encodeURIComponent(publicClassId)}/book`, {
      method: "POST",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey },
      body: JSON.stringify({ guardianName, guardianEmail })
    })
  );
}
