import { useCallback, useEffect, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { invoke } from "@/shared/api/invoke";
import { Button, LinkButton } from "@/ui";

type BackendStatus = "checking" | "ok" | "error";

export function HomePage() {
  const { t } = useI18n();
  const [status, setStatus] = useState<BackendStatus>("checking");
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const checkBackend = useCallback(async () => {
    setStatus("checking");
    setStatusMessage(null);

    try {
      const message = await invoke<string>("app_status");
      setStatus("ok");
      setStatusMessage(message);
    } catch {
      setStatus("error");
      setStatusMessage(null);
    }
  }, []);

  useEffect(() => {
    void checkBackend();
  }, [checkBackend]);

  return (
    <main className="flex min-h-full items-center justify-center p-8">
      <section className="w-full max-w-lg rounded-md border border-border bg-surface p-8 shadow-sm">
        <p className="text-sm font-medium uppercase tracking-wide text-muted">
          {t("home.title")}
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-foreground">
          {t("app.name")}
        </h1>
        <p className="mt-3 text-lg text-muted">{t("app.running")}</p>

        <div className="mt-6 flex flex-wrap gap-3">
          <LinkButton to="/repairs/intake" variant="primary">
            {t("repairs.intake.start")}
          </LinkButton>
          <LinkButton to="/search" variant="secondary">
            {t("nav.search")}
          </LinkButton>
        </div>

        <div className="mt-6 flex flex-col gap-3">
          <p className="text-sm text-foreground">
            {status === "checking" && t("app.backendChecking")}
            {status === "ok" && t("app.backendOk")}
            {status === "error" && t("app.backendError")}
          </p>
          {statusMessage ? (
            <p className="font-mono text-xs text-muted">{statusMessage}</p>
          ) : null}
          <Button type="button" onClick={() => void checkBackend()}>
            {t("home.refresh")}
          </Button>
        </div>
      </section>
    </main>
  );
}
