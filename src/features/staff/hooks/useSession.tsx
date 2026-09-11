import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { staffApi } from "@/features/staff/api/staffApi";
import type { Session } from "@/features/staff/types/staff";
import {
  SessionContext,
  type SessionContextValue,
} from "@/features/staff/hooks/session-context";
import { subscribeUnauthorized } from "@/shared/api/invoke";

export { useSession } from "@/features/staff/hooks/session-context";

export function SessionProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await staffApi.getCurrentSession();
      setSession(next);
    } catch (err) {
      setSession(null);
      setError(err instanceof Error ? err.message : "Failed to load session");
    } finally {
      setLoading(false);
    }
  }, []);

  const applySession = useCallback((next: Session) => {
    setSession(next);
    setError(null);
    setLoading(false);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    return subscribeUnauthorized(() => {
      void refresh();
    });
  }, [refresh]);

  const value = useMemo<SessionContextValue>(
    () => ({
      session,
      loading,
      error,
      refresh,
      applySession,
    }),
    [session, loading, error, refresh, applySession],
  );

  return (
    <SessionContext.Provider value={value}>{children}</SessionContext.Provider>
  );
}
