export const AUTO_BACKUP_INTERVALS = [
  "never",
  "day",
  "week",
  "month",
  "year",
] as const;

export type AutoBackupInterval = (typeof AUTO_BACKUP_INTERVALS)[number];

export type AutoBackupSettings = {
  interval: AutoBackupInterval;
  folderPath: string | null;
};

export function isAutoBackupInterval(
  value: string,
): value is AutoBackupInterval {
  return (AUTO_BACKUP_INTERVALS as readonly string[]).includes(value);
}
