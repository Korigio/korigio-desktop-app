import { useForm } from "@tanstack/react-form";
import { useRef } from "react";
import { useNavigate } from "react-router";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company, CompanyInput } from "@/features/companies/types/company";

type Options = {
  company?: Company;
  mode: "create" | "edit";
  onSuccess?: (entity: Company) => void;
  navigateOnSuccess?: boolean;
};

function toInput(company?: Company): CompanyInput {
  return {
    legalName: company?.legalName ?? "",
    tradeName: company?.tradeName ?? "",
    taxId: company?.taxId ?? "",
    address: company?.address ?? "",
    phone: company?.phone ?? "",
    email: company?.email ?? "",
    website: company?.website ?? "",
  };
}

export function useCompanyForm({
  company,
  mode,
  onSuccess,
  navigateOnSuccess = true,
}: Options) {
  const navigate = useNavigate();
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const navigateOnSuccessRef = useRef(navigateOnSuccess);
  navigateOnSuccessRef.current = navigateOnSuccess;

  const form = useForm({
    defaultValues: toInput(company),
    onSubmit: async ({ value }) => {
      const input: CompanyInput = {
        legalName: value.legalName,
        tradeName: value.tradeName || null,
        taxId: value.taxId || null,
        address: value.address || null,
        phone: value.phone || null,
        email: value.email || null,
        website: value.website || null,
      };

      if (mode === "create") {
        const created = await companiesApi.create(input);
        onSuccessRef.current?.(created);
        if (navigateOnSuccessRef.current) {
          navigate(`/companies/${created.id}`);
        }
        return;
      }

      if (!company) {
        throw new Error("Missing company for edit");
      }
      const updated = await companiesApi.update(company.id, input);
      onSuccessRef.current?.(updated);
      if (navigateOnSuccessRef.current) {
        navigate(`/companies/${updated.id}`);
      }
    },
  });

  return form;
}
