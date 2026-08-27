export type BackupKind = "manual" | "auto" | "safety";

export type BackupInfo = {
  path: string;
  fileName: string;
  kind: BackupKind;
  createdAt: string;
  sizeBytes: number;
};

export type CreateBackupInput = {
  destinationPath?: string | null;
};

export type BackupValidationResult = {
  valid: boolean;
  appVersion: string | null;
  createdAt: string | null;
  errors: string[];
};

export type RestoreBackupResult = {
  restoredFrom: string;
  safetyBackupPath: string;
};

export type LocalBackupListResult = {
  items: BackupInfo[];
};

export type AutoBackupResult = {
  ran: boolean;
  backup: BackupInfo | null;
  prunedCount: number;
};
