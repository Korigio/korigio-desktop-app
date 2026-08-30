/** Short weekday for a `YYYY-MM-DD` date. Invalid dates return the raw string. */
export function weekdayLabel(date: string, locale: string): string {
  const parts = date.split("-").map(Number);
  const year = parts[0];
  const month = parts[1];
  const day = parts[2];
  if (!year || !month || !day) {
    return date;
  }
  return new Intl.DateTimeFormat(locale, { weekday: "short" }).format(
    new Date(year, month - 1, day),
  );
}
