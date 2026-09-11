import { useEffect, useState } from "react";
import { Navigate } from "react-router";
import { companiesApi } from "@/features/companies/api/companiesApi";
import { FirstRunSetupPage } from "@/features/setup/pages/FirstRunSetupPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { StatusMessage } from "@/ui";

type GateState = "checking" | "setup" | "home" | "error";

export default function SetupRoute() {
  const { t } = useI18n();
  const [state, setState] = useState<GateState>("checking");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void companiesApi
      .hasActiveCompanies()
      .then((has) => {
        if (!cancelled) {
          setError(null);
          setState(has ? "home" : "setup");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err ?? "Error"));
          setState("error");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (state === "checking") {
    return (
      <main className="flex min-h-full items-center justify-center p-8">
        <StatusMessage>{t("common.loading")}</StatusMessage>
      </main>
    );
  }

  if (state === "error") {
    return (
      <main className="flex min-h-full items-center justify-center p-8">
        <StatusMessage tone="danger">{error}</StatusMessage>
      </main>
    );
  }

  if (state === "home") {
    return <Navigate to="/" replace />;
  }

  return <FirstRunSetupPage />;
}
