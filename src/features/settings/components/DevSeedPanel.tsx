import { useState } from "react";
import { seedApi } from "@/features/settings/api/seedApi";
import { Button, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

/** Debug-only synthetic load tool (Phase 13). Hidden in production builds. */
export function DevSeedPanel() {
  const { t } = useI18n();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  if (!import.meta.env.DEV) {
    return null;
  }

  return (
    <section className="mt-8 flex max-w-2xl flex-col gap-3">
      <PageHeader
        title={t("settings.devSeed.title")}
        description={t("settings.devSeed.subtitle")}
      />
      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
      {success ? <StatusMessage tone="success">{success}</StatusMessage> : null}
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          disabled={busy}
          onClick={() => {
            void (async () => {
              setBusy(true);
              setError(null);
              setSuccess(null);
              try {
                const result = await seedApi.run({
                  customers: 100,
                  devices: 200,
                  repairs: 500,
                  confirm: true,
                });
                setSuccess(
                  t("settings.devSeed.success")
                    .replace("{customers}", String(result.customers))
                    .replace("{devices}", String(result.devices))
                    .replace("{repairs}", String(result.repairs))
                    .replace("{ms}", String(result.elapsedMs)),
                );
              } catch (err) {
                setError(
                  err instanceof Error
                    ? err.message
                    : t("settings.devSeed.error"),
                );
              } finally {
                setBusy(false);
              }
            })();
          }}
        >
          {busy ? t("common.saving") : t("settings.devSeed.actions.small")}
        </Button>
        <Button
          type="button"
          variant="secondary"
          disabled={busy}
          onClick={() => {
            const confirmed = window.confirm(t("settings.devSeed.fullConfirm"));
            if (!confirmed) {
              return;
            }
            void (async () => {
              setBusy(true);
              setError(null);
              setSuccess(null);
              try {
                const result = await seedApi.run({
                  customers: 10_000,
                  devices: 20_000,
                  repairs: 50_000,
                  confirm: true,
                });
                setSuccess(
                  t("settings.devSeed.success")
                    .replace("{customers}", String(result.customers))
                    .replace("{devices}", String(result.devices))
                    .replace("{repairs}", String(result.repairs))
                    .replace("{ms}", String(result.elapsedMs)),
                );
              } catch (err) {
                setError(
                  err instanceof Error
                    ? err.message
                    : t("settings.devSeed.error"),
                );
              } finally {
                setBusy(false);
              }
            })();
          }}
        >
          {t("settings.devSeed.actions.full")}
        </Button>
      </div>
    </section>
  );
}
