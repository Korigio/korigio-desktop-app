import { BackupPanel } from "@/features/settings/components/BackupPanel";
import { ScheduledBackupSettings } from "@/features/settings/components/ScheduledBackupSettings";

export function BackupSettingsPage() {
  return (
    <div className="flex flex-col gap-4">
      <ScheduledBackupSettings />
      <BackupPanel />
    </div>
  );
}
