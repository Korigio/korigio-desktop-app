import { useCallback, useEffect, useState } from "react";
import { teamApi } from "@/features/team/api/teamApi";
import type {
  NearbyTeam,
  SyncStatus,
  Team,
  TeamMember,
} from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

const POLL_MS = 2000;

export function useTeamSidebar() {
  const { t } = useI18n();
  const [team, setTeam] = useState<Team | null>(null);
  const [pin, setPin] = useState<string | null>(null);
  const [members, setMembers] = useState<TeamMember[]>([]);
  const [nearby, setNearby] = useState<NearbyTeam[]>([]);
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await teamApi.get();
      setTeam(next);
      if (!next) {
        setPin(null);
        setMembers([]);
        setSyncStatus(null);
        return;
      }
      try {
        const pinResult = await teamApi.getPin();
        setPin(pinResult.pin);
      } catch {
        setPin(null);
      }
      try {
        const memberResult = await teamApi.listMembers();
        setMembers(memberResult.items);
      } catch {
        setMembers([]);
      }
    } catch (err) {
      setTeam(null);
      setPin(null);
      setMembers([]);
      setSyncStatus(null);
      setError(err instanceof Error ? err.message : t("team.notInTeam"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  const silentRefresh = useCallback(async () => {
    try {
      const next = await teamApi.get();
      setTeam(next);
      if (!next) {
        setPin(null);
        setMembers([]);
        setSyncStatus(null);
        return;
      }
      try {
        const pinResult = await teamApi.getPin();
        setPin(pinResult.pin);
      } catch {
        // Keep the last successful PIN.
      }
      try {
        const memberResult = await teamApi.listMembers();
        setMembers(memberResult.items);
      } catch {
        // Keep the last successful roster.
      }
    } catch {
      // Keep the last successful team snapshot.
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const refreshSyncStatus = useCallback(async () => {
    try {
      const next = await teamApi.getSyncStatus();
      setSyncStatus(next);
    } catch {
      // Keep the last successful sync status.
    }
  }, []);

  useSyncApplied(() => {
    void silentRefresh();
    void refreshSyncStatus();
  });

  useEffect(() => {
    if (team) {
      setNearby([]);
      return;
    }
    let cancelled = false;
    async function tick() {
      try {
        const result = await teamApi.listNearby();
        if (!cancelled) {
          setNearby(result.items);
        }
      } catch {
        // Keep the last successful nearby list until the next tick.
      }
    }
    void tick();
    const id = window.setInterval(() => {
      void tick();
    }, POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [team]);

  const inTeam = team !== null;

  useEffect(() => {
    if (!inTeam) {
      setSyncStatus(null);
      return;
    }
    let cancelled = false;
    async function tick() {
      try {
        const next = await teamApi.getSyncStatus();
        if (!cancelled) {
          setSyncStatus(next);
        }
      } catch {
        // Keep the last successful sync status.
      }
    }
    void tick();
    const id = window.setInterval(() => {
      void tick();
    }, POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [inTeam]);

  return {
    team,
    pin,
    members,
    nearby,
    syncStatus,
    loading,
    error,
    setError,
    setPin,
    reload,
  };
}
