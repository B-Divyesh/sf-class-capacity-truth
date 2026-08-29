import { accessToken } from "./auth";

export type Availability = "available" | "full" | "cutoff";
export interface ClassSession { publicId: string; name: string; startsAt: number; bookingCutoff: number; timezone: string; capacity: number; confirmed: number; openSeats: number; availability: Availability }
export interface DemoData { schoolName: string; expiresAt: number; classes: ClassSession[] }
export interface RealClass extends ClassSession { id: string; published: boolean; calendarConfirmed: number | null; reconciliationStatus: "matched" | "attention" | null }
export interface Workspace { id: string; schoolName: string; subscriptionStatus: "trial" | "active" | "grace" | "inactive"; trialEndsAt: number | null }
export interface BookingSummary { id: string; guardianName: string; guardianEmail: string; createdAt: number }
export interface RuntimeStatus { emailDelivery: "smtp" | "not_configured" }
export type OfferDeliveryStatus = "ready_to_copy" | "email_queued" | "email_sent" | "email_failed" | "accepted" | "expired" | "legacy_recorded";
export interface OfferReceipt { id: string; classId: string; className: string; recipientName: string; offerUrl: string; expiresAt: number; offerStatus: string; deliveryStatus: OfferDeliveryStatus; createdAt: number }
export interface ReleaseResult { offerToken: string | null; offerUrl: string | null; expiresAt: number | null; deliveryStatus: "ready_to_copy" | "email_queued" | "not_needed" }

interface ApiErrorBody { code?: string; message?: string }
export class ApiError extends Error {
  constructor(message: string, public readonly status: number, public readonly code?: string) { super(message); }
}
async function responseJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as ApiErrorBody;
    throw new ApiError(body.message ?? "The request did not finish. Try again.", response.status, body.code);
  }
  if (response.status === 204 || response.headers.get("content-length") === "0") return undefined as T;
  return (await response.json()) as T;
}
async function workspaceHeaders(json = false): Promise<Record<string, string>> {
  const token = await accessToken();
  const key = localStorage.getItem("cct:workspace-key");
  return {
    ...(json ? { "Content-Type": "application/json" } : {}),
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(key ? { "X-Workspace-Key": key } : {})
  };
}

export function workspaceKey() { return localStorage.getItem("cct:workspace-key"); }
export function clearWorkspaceKey() { localStorage.removeItem("cct:workspace-key"); }
export async function createWorkspace(schoolName: string) {
  const result = await responseJson<{ workspace: Workspace; accessKey: string }>(await fetch("/api/workspaces", { method: "POST", headers: await workspaceHeaders(true), body: JSON.stringify({ schoolName }) }));
  localStorage.setItem("cct:workspace-key", result.accessKey);
  return result.workspace;
}
export async function loadWorkspace() { const result = await responseJson<{ workspace: Workspace; accessKey: string }>(await fetch("/api/workspaces", { headers: await workspaceHeaders() })); localStorage.setItem("cct:workspace-key", result.accessKey); return result.workspace; }
export async function loadRuntimeStatus() { return responseJson<RuntimeStatus>(await fetch("/api/runtime")); }
export async function listClasses() { return responseJson<RealClass[]>(await fetch("/api/workspaces/classes", { headers: await workspaceHeaders() })); }
export async function createClass(input: { name: string; startsAt: number; bookingCutoff: number; timezone: string; capacity: number }) { return responseJson<RealClass>(await fetch("/api/workspaces/classes", { method: "POST", headers: await workspaceHeaders(true), body: JSON.stringify(input) })); }
export async function publishClass(id: string) { return responseJson<RealClass>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/publish`, { method: "POST", headers: await workspaceHeaders() })); }
export async function connectCalendar(label: string, feedUrl: string) { return responseJson<{ label: string; provider: string; enabled: boolean }>(await fetch("/api/workspaces/calendar", { method: "PUT", headers: await workspaceHeaders(true), body: JSON.stringify({ label, feedUrl }) })); }
export async function checkCalendar() { return responseJson<{ checked: number }>(await fetch("/api/workspaces/calendar/check", { method: "POST", headers: await workspaceHeaders() })); }
export async function reconcileClass(id: string, calendarConfirmed: number) { return responseJson<RealClass>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/reconcile`, { method: "POST", headers: await workspaceHeaders(true), body: JSON.stringify({ calendarConfirmed }) })); }
export async function listBookings(id: string) { return responseJson<BookingSummary[]>(await fetch(`/api/workspaces/classes/${encodeURIComponent(id)}/bookings`, { headers: await workspaceHeaders() })); }
export async function cancelBooking(classId: string, bookingId: string) { return responseJson<ReleaseResult>(await fetch(`/api/workspaces/classes/${encodeURIComponent(classId)}/bookings/${encodeURIComponent(bookingId)}/cancel`, { method: "POST", headers: await workspaceHeaders() })); }
export async function listOfferReceipts() { return responseJson<OfferReceipt[]>(await fetch("/api/workspaces/offers", { headers: await workspaceHeaders() })); }
export async function exportWorkspace() { return responseJson<unknown>(await fetch("/api/workspaces/export", { headers: await workspaceHeaders() })); }
export async function deleteWorkspace() { return responseJson<void>(await fetch("/api/workspaces/data", { method: "DELETE", headers: await workspaceHeaders() })); }
export async function verifyBilling(license: string) { return responseJson<Workspace>(await fetch("/api/workspaces/billing/verify", { method: "POST", headers: await workspaceHeaders(true), body: JSON.stringify({ license }) })); }

export async function createCheckoutSession() {
  const result = await responseJson<{ checkout_url: string }>(await fetch("https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: "{}"
  }));
  const url = new URL(result.checkout_url);
  if (url.protocol !== "https:" || url.hostname !== "checkout.dodopayments.com") throw new Error("The checkout returned an unexpected address. Try again.");
  return url.toString();
}

export async function loadPublicClass(publicId: string) { return responseJson<RealClass>(await fetch(`/api/classes/${encodeURIComponent(publicId)}`)); }
export async function bookRealClass(publicId: string, guardianName: string, guardianEmail: string, idempotencyKey: string) { return responseJson<RealClass>(await fetch(`/api/classes/${encodeURIComponent(publicId)}/book`, { method: "POST", headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey }, body: JSON.stringify({ guardianName, guardianEmail }) })); }
export async function joinWaitlist(publicId: string, guardianName: string, guardianEmail: string, idempotencyKey: string) { return responseJson<{ waitlistId: string; status: string }>(await fetch(`/api/classes/${encodeURIComponent(publicId)}/waitlist`, { method: "POST", headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey }, body: JSON.stringify({ guardianName, guardianEmail, consent: true }) })); }
export async function loadOffer(token: string) { return responseJson<{ offerToken: string; class: RealClass; expiresAt: number }>(await fetch(`/api/offers/${encodeURIComponent(token)}`)); }
export async function acceptOffer(token: string) { return responseJson<RealClass>(await fetch(`/api/offers/${encodeURIComponent(token)}/accept`, { method: "POST" })); }

export async function loadDemo(signal?: AbortSignal): Promise<DemoData> { return responseJson<DemoData>(await fetch("/api/demo/session", { credentials: "same-origin", signal })); }
export async function resetDemo(): Promise<DemoData> { return responseJson<DemoData>(await fetch("/api/demo/reset", { method: "POST", credentials: "same-origin" })); }
export async function leaveDemo(): Promise<void> { return responseJson<void>(await fetch("/api/demo/leave", { method: "POST", credentials: "same-origin" })); }
export async function bookSeat(publicClassId: string, guardianName: string, guardianEmail: string, idempotencyKey: string) { return responseJson<{ bookingId: string; class: ClassSession; repeated: boolean }>(await fetch(`/api/demo/classes/${encodeURIComponent(publicClassId)}/book`, { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey }, body: JSON.stringify({ guardianName, guardianEmail }) })); }
