import type { DiagnosisResultItem } from "@/features/diagnosis/types/diagnosis";

/** Build note lines from ephemeral checklist values (checked / non-empty text). */
export function checklistItemsToNoteLines(
  items: DiagnosisResultItem[],
): string[] {
  const lines: string[] = [];
  for (const item of items) {
    if (item.kind === "checkbox") {
      if (item.value === true) {
        lines.push(`✓ ${item.label}`);
      }
      continue;
    }
    const text = typeof item.value === "string" ? item.value.trim() : "";
    if (text.length > 0) {
      lines.push(`${item.label}: ${text}`);
    }
  }
  return lines;
}

/** Append checklist-derived lines to notes; blank line separator if notes non-empty. */
export function appendChecklistToNotes(
  notes: string,
  items: DiagnosisResultItem[],
): string {
  const lines = checklistItemsToNoteLines(items);
  if (lines.length === 0) return notes;
  const block = lines.join("\n");
  const trimmed = notes.trimEnd();
  if (trimmed.length === 0) return block;
  return `${trimmed}\n\n${block}`;
}

export function checklistHasApplicableValues(
  items: DiagnosisResultItem[],
): boolean {
  return checklistItemsToNoteLines(items).length > 0;
}
