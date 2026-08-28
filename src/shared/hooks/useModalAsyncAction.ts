import { useEffect, useState } from "react";

/** Busy/error state for modal confirm actions; clears error when the modal closes. */
export function useModalAsyncAction(isOpen: boolean) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      setError(null);
    }
  }, [isOpen]);

  async function run<T>(
    action: () => Promise<T>,
    fallbackMessage: string,
  ): Promise<T | null> {
    setError(null);
    setBusy(true);
    try {
      return await action();
    } catch (err) {
      setError(err instanceof Error ? err.message : fallbackMessage);
      return null;
    } finally {
      setBusy(false);
    }
  }

  return { busy, error, setError, run };
}
