import { useForm } from "@tanstack/react-form";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { CompanyInput } from "@/features/companies/types/company";
import { settingsApi } from "@/features/settings/api/settingsApi";
import { isCommandError } from "@/shared/api/invoke";
import { useI18n } from "@/shared/hooks/useI18n";

const FALLBACK_TAX_RATE_PERCENT = "19";
const FALLBACK_CURRENCY = "EUR";

export type FirstRunFormValues = {
  legalName: string;
  tradeName: string;
  taxId: string;
  address: string;
  phone: string;
  email: string;
  website: string;
  taxRatePercent: string;
  currency: string;
};

function toCompanyInput(value: FirstRunFormValues): CompanyInput {
  return {
    legalName: value.legalName,
    tradeName: value.tradeName || null,
    taxId: value.taxId || null,
    address: value.address || null,
    phone: value.phone || null,
    email: value.email || null,
    website: value.website || null,
  };
}

export function useFirstRunForm() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);

  const form = useForm({
    defaultValues: {
      legalName: "",
      tradeName: "",
      taxId: "",
      address: "",
      phone: "",
      email: "",
      website: "",
      taxRatePercent: FALLBACK_TAX_RATE_PERCENT,
      currency: FALLBACK_CURRENCY,
    } satisfies FirstRunFormValues,
    onSubmit: async ({ value }) => {
      setError(null);
      try {
        const created = await companiesApi.create(toCompanyInput(value));
        if (!created.isDefault) {
          try {
            await companiesApi.setDefault(created.id);
          } catch {
            // Default is best-effort; an active company is enough to leave setup.
          }
        }
        try {
          await settingsApi.setShopSettings({
            taxRatePercent: value.taxRatePercent,
            currency: value.currency,
          });
        } catch {
          // Shop save is best-effort; company exists so setup can continue.
        }
        navigate("/");
      } catch (err) {
        setError(
          isCommandError(err) ? err.message : t("setup.errors.submitFailed"),
        );
      }
    },
  });

  useEffect(() => {
    let cancelled = false;
    void settingsApi
      .getShopSettings()
      .then((shop) => {
        if (cancelled) {
          return;
        }
        form.setFieldValue("taxRatePercent", shop.taxRatePercent);
        form.setFieldValue("currency", shop.currency);
      })
      .catch(() => {
        // Keep 19 / EUR so setup is not blocked.
      })
      .finally(() => {
        if (!cancelled) {
          setReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [form]);

  return { form, error, ready };
}
