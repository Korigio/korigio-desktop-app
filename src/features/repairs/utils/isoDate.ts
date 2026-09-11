/** Local calendar date as `YYYY-MM-DD` (UTC slice of ISO timestamp). */
export function todayIsoDate(): string {
  return new Date().toISOString().slice(0, 10);
}

/** Prefer existing `collectedAt` (date part); otherwise today. */
export function collectedAtOrToday(collectedAt: string | null): string {
  const datePart = collectedAt?.slice(0, 10)?.trim();
  return datePart || todayIsoDate();
}
