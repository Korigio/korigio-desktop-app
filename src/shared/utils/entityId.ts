/** UUID v7: version nibble 7, RFC variant 8/9/a/b. */
const ENTITY_ID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function isEntityId(s: string): boolean {
  return ENTITY_ID_RE.test(s.trim());
}

export function parseEntityId(raw: string | undefined): string | null {
  if (raw == null) {
    return null;
  }
  const trimmed = raw.trim();
  if (!isEntityId(trimmed)) {
    return null;
  }
  return trimmed.toLowerCase();
}
