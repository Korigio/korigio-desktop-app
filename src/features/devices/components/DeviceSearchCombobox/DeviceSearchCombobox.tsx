import type { DeviceSearchComboboxState } from "@/features/devices/hooks/useDeviceSearchCombobox";
import { deviceLabel } from "@/features/devices/types/device";
import { SearchCombobox } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  combobox: DeviceSearchComboboxState;
  disabled?: boolean;
  quickCreateLabel?: string;
  onQuickCreate?: () => void;
};

export function DeviceSearchCombobox({
  combobox,
  disabled = false,
  quickCreateLabel,
  onQuickCreate,
}: Props) {
  const { t } = useI18n();

  return (
    <SearchCombobox
      label={t("repairs.fields.device")}
      query={combobox.query}
      onQueryChange={combobox.setQuery}
      items={combobox.items}
      renderItem={deviceLabel}
      getItemKey={(item) => item.id}
      onSelect={combobox.select}
      selectedLabel={combobox.selectedLabel}
      onClearSelection={combobox.clear}
      loading={combobox.loading}
      disabled={disabled}
      emptyMessage={
        disabled
          ? t("repairs.intake.selectCustomerFirst")
          : t("repairs.intake.noDevices")
      }
      footerAction={
        !disabled && quickCreateLabel && onQuickCreate
          ? { label: quickCreateLabel, onClick: onQuickCreate }
          : undefined
      }
    />
  );
}
