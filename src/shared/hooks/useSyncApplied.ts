import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

const SYNC_APPLIED_EVENT = "sync-applied";
const DEBOUNCE_MS = 250;

export type SyncAppliedPayload = {
  at: string;
  applied: number;
};

/**
 * Subscribes to gossip `sync-applied` once per mount.
 * Bursts are collapsed with a 250ms debounce.
 */
export function useSyncApplied(onApplied: () => void) {
  const onAppliedRef = useRef(onApplied);
  onAppliedRef.current = onApplied;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let timer: number | undefined;

    const schedule = () => {
      if (timer !== undefined) {
        window.clearTimeout(timer);
      }
      timer = window.setTimeout(() => {
        timer = undefined;
        onAppliedRef.current();
      }, DEBOUNCE_MS);
    };

    void listen<SyncAppliedPayload>(SYNC_APPLIED_EVENT, () => {
      schedule();
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((error: unknown) => {
        console.error("Failed to subscribe to sync-applied event", error);
      });

    return () => {
      if (timer !== undefined) {
        window.clearTimeout(timer);
      }
      unlisten?.();
    };
  }, []);
}
