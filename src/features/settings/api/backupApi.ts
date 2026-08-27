import { invoke } from "@/shared/api/invoke";
import type {
  AutoBackupResult,
  BackupInfo,
  BackupValidationResult,
  CreateBackupInput,
  LocalBackupListResult,
  RestoreBackupResult,
} from "@/features/settings/types/backup";

export const backupApi = {
  create(input: CreateBackupInput = {}): Promise<BackupInfo> {
    return invoke<BackupInfo>("create_backup", { input });
  },
  validate(path: string): Promise<BackupValidationResult> {
    return invoke<BackupValidationResult>("validate_backup", { path });
  },
  restore(path: string): Promise<RestoreBackupResult> {
    return invoke<RestoreBackupResult>("restore_backup", { path });
  },
  listLocal(): Promise<LocalBackupListResult> {
    return invoke<LocalBackupListResult>("list_local_backups");
  },
  runAutoIfDue(): Promise<AutoBackupResult> {
    return invoke<AutoBackupResult>("run_auto_backup_if_due");
  },
};
