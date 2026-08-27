import { useEffect, useRef } from "react";
import { backupApi } from "@/features/settings/api/backupApi";

/** Fire-and-forget daily auto-backup once per app shell mount. */
export function AutoBackupOnStartup() {
  const started = useRef(false);

  useEffect(() => {
    if (started.current) {
      return;
    }
    started.current = true;
    void backupApi.runAutoIfDue().catch(() => {
      // Non-blocking; backup failures must not break the shell.
    });
  }, []);

  return null;
}
