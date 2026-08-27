import type { CustomerSearchComboboxState } from "@/features/customers/hooks/useCustomerSearchCombobox";
import { customerLabel } from "@/features/customers/hooks/useCustomerSearchCombobox";
import { SearchCombobox } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  combobox: CustomerSearchComboboxState;
  quickCreateLabel?: string;
  onQuickCreate?: () => void;
};

export function CustomerSearchCombobox({
  combobox,
  quickCreateLabel,
  onQuickCreate,
}: Props) {
  const { t } = useI18n();

  return (
    <SearchCombobox
      inputRef={combobox.inputRef}
      label={t("repairs.fields.customer")}
      query={combobox.query}
      onQueryChange={combobox.setQuery}
      items={combobox.items}
      renderItem={customerLabel}
      getItemKey={(item) => item.id}
      onSelect={combobox.select}
      selectedLabel={combobox.selectedLabel}
      onClearSelection={combobox.clear}
      loading={combobox.loading}
      emptyMessage={t("repairs.intake.noCustomers")}
      footerAction={
        quickCreateLabel && onQuickCreate
          ? { label: quickCreateLabel, onClick: onQuickCreate }
          : undefined
      }
    />
  );
}
