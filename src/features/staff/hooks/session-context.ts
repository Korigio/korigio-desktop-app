import { createContext, useContext } from "react";
import type { Session } from "@/features/staff/types/staff";

export type SessionContextValue = {
  session: Session | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  applySession: (next: Session) => void;
};

export const SessionContext = createContext<SessionContextValue | null>(null);

export function useSession(): SessionContextValue {
  const context = useContext(SessionContext);
  if (!context)
    throw new Error("useSession must be used within SessionProvider");
  return context;
}
