import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

type Options = {
  company: Company;
  onCompanyChange: (company: Company) => void;
  disabled?: boolean;
};

export function useCompanyLogo({
  company,
  onCompanyChange,
  disabled = false,
}: Options) {
  const { t } = useI18n();
  const [logoUrl, setLogoUrl] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reloadLogoUrl = useCallback(async () => {
    if (!company.logoPath) {
      setLogoUrl(null);
      return;
    }
    try {
      const resolved = await companiesApi.resolveLogoPath(company.id);
      setLogoUrl(convertFileSrc(resolved.absolutePath));
    } catch {
      setLogoUrl(null);
    }
  }, [company.id, company.logoPath]);

  useEffect(() => {
    void reloadLogoUrl();
  }, [reloadLogoUrl]);

  const silentRefresh = useCallback(async () => {
    try {
      const next = await companiesApi.get(company.id);
      onCompanyChange(next);
      if (!next.logoPath) {
        setLogoUrl(null);
        return;
      }
      const resolved = await companiesApi.resolveLogoPath(next.id);
      setLogoUrl(convertFileSrc(resolved.absolutePath));
    } catch {
      // Keep the last resolved logo.
    }
  }, [company.id, onCompanyChange]);

  useSyncApplied(() => {
    void silentRefresh();
  });

  const uploadLogo = useCallback(async () => {
    if (disabled) {
      return;
    }
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Images",
            extensions: ["jpg", "jpeg", "png", "webp"],
          },
        ],
      });
      if (selected === null || Array.isArray(selected)) {
        return;
      }

      setBusy(true);
      const updated = await companiesApi.attachLogo({
        companyId: company.id,
        sourcePath: selected,
      });
      onCompanyChange(updated);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("companies.logo.attachFailed"),
      );
    } finally {
      setBusy(false);
    }
  }, [company.id, disabled, onCompanyChange, t]);

  const clearLogo = useCallback(async () => {
    if (disabled) {
      return;
    }
    setError(null);
    setBusy(true);
    try {
      const updated = await companiesApi.clearLogo(company.id);
      onCompanyChange(updated);
      setLogoUrl(null);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("companies.logo.clearFailed"),
      );
    } finally {
      setBusy(false);
    }
  }, [company.id, disabled, onCompanyChange, t]);

  return {
    logoUrl,
    busy,
    error,
    hasLogo: Boolean(company.logoPath),
    uploadLogo,
    clearLogo,
  };
}
