import { useCallback, useState } from "react";
import { useNavigate } from "react-router";
import { useI18n } from "@/shared/hooks/useI18n";

type Options = {
  repairId: string;
  closeLabelKey: string;
};

/** Tauri macOS overrides `window.print` to return an invoke Promise. */
async function tryWindowPrint(): Promise<void> {
  const result = window.print() as unknown;
  if (
    result != null &&
    typeof result === "object" &&
    "then" in result &&
    typeof (result as PromiseLike<unknown>).then === "function"
  ) {
    await result;
  }
}

export function usePrintPageActions({ repairId, closeLabelKey }: Options) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const print = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      await tryWindowPrint();
    } catch {
      setError(t("print.errors.printFailed"));
    } finally {
      setBusy(false);
    }
  }, [t]);

  const close = useCallback(() => {
    setError(null);
    navigate(`/repairs/${repairId}`);
  }, [navigate, repairId]);

  return {
    print,
    close,
    busy,
    error,
    closeLabel: t(closeLabelKey),
  };
}
