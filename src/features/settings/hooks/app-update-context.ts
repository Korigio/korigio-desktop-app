import { createContext, useContext } from "react";

export type AppUpdateContextValue = {
  updateAvailable: boolean;
  currentVersion: string | null;
  latestVersion: string | null;
  downloadUrl: string | null;
  checking: boolean;
  error: string | null;
  checkAgain: () => Promise<void>;
};

export const AppUpdateContext = createContext<AppUpdateContextValue | null>(
  null,
);

export function useAppUpdate(): AppUpdateContextValue {
  const context = useContext(AppUpdateContext);
  if (!context)
    throw new Error("useAppUpdate must be used within AppUpdateProvider");
  return context;
}
