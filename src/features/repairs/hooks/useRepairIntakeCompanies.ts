import { useCallback, useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import type { IntakeGate } from "@/features/repairs/types/repairIntake";
import { preferredCompany } from "@/features/repairs/utils/repairIntake";

export function useRepairIntakeCompanies(onSelect: () => void) {
  const [gate, setGate] = useState<IntakeGate>("loading");
  const [companies, setCompanies] = useState<Company[]>([]);
  const [company, setCompany] = useState<Company | null>(null);
  const [companiesError, setCompaniesError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void companiesApi
      .list({ includeArchived: false, page: 1, pageSize: 100 })
      .then((result) => {
        if (cancelled) return;
        setCompanies(result.items);
        setCompany(preferredCompany(result.items));
        setGate(result.items.length === 0 ? "no-company" : "ready");
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        setCompanies([]);
        setCompany(null);
        setCompaniesError(
          error instanceof Error ? error.message : "Failed to load companies",
        );
        setGate("no-company");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const selectCompany = useCallback(
    (next: Company) => {
      setCompany(next);
      onSelect();
    },
    [onSelect],
  );

  const resetCompany = useCallback(
    () => setCompany(preferredCompany(companies)),
    [companies],
  );

  return {
    gate,
    companies,
    companiesError,
    company,
    selectCompany,
    resetCompany,
  };
}
