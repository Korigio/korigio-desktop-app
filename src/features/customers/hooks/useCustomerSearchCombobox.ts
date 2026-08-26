import { useCallback } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { useAsyncSearchCombobox } from "@/shared/hooks/useAsyncSearchCombobox";

export function customerLabel(customer: Customer): string {
  return customer.phone
    ? `${customer.name} · ${customer.phone}`
    : customer.name;
}

type Options = {
  onSelect?: (customer: Customer) => void;
  onClear?: () => void;
};

export function useCustomerSearchCombobox(options: Options = {}) {
  const searchCustomers = useCallback(async (query: string) => {
    const result = await customersApi.list({
      query: query.trim() || undefined,
      includeArchived: false,
      page: 1,
      pageSize: 20,
    });
    return result.items;
  }, []);

  return useAsyncSearchCombobox({
    search: searchCustomers,
    getLabel: customerLabel,
    onSelect: options.onSelect,
    onClear: options.onClear,
  });
}

export type CustomerSearchComboboxState = ReturnType<
  typeof useCustomerSearchCombobox
>;
