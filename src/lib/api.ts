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
