import { useCallback } from "react";
import { devicesApi } from "@/features/devices/api/devicesApi";
import { deviceLabel, type Device } from "@/features/devices/types/device";
import { useAsyncSearchCombobox } from "@/shared/hooks/useAsyncSearchCombobox";

type Options = {
  customerId?: number;
  onSelect?: (device: Device) => void;
  onClear?: () => void;
};

export function useDeviceSearchCombobox({
  customerId,
  onSelect,
  onClear,
}: Options) {
  const searchDevices = useCallback(
    async (query: string) => {
      if (!customerId) {
        return [];
      }
      const result = await devicesApi.list({
        query: query.trim() || undefined,
        customerId,
        includeArchived: false,
        page: 1,
        pageSize: 20,
      });
      return result.items;
    },
    [customerId],
  );

  return useAsyncSearchCombobox({
    search: searchDevices,
    getLabel: deviceLabel,
    enabled: Boolean(customerId),
    onSelect,
    onClear,
  });
}

export type DeviceSearchComboboxState = ReturnType<
  typeof useDeviceSearchCombobox
>;
