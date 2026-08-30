import { invoke } from "@/shared/api/invoke";
import type {
  Device,
  DeviceInput,
  DeviceListQuery,
  DeviceListResult,
} from "@/features/devices/types/device";

export const devicesApi = {
  list(query: DeviceListQuery = {}): Promise<DeviceListResult> {
    return invoke<DeviceListResult>("list_devices", { query });
  },
  get(id: string): Promise<Device> {
    return invoke<Device>("get_device", { id });
  },
  create(input: DeviceInput): Promise<Device> {
    return invoke<Device>("create_device", { input });
  },
  update(id: string, input: DeviceInput): Promise<Device> {
    return invoke<Device>("update_device", { id, input });
  },
  archive(id: string): Promise<Device> {
    return invoke<Device>("archive_device", { id });
  },
  unarchive(id: string): Promise<Device> {
    return invoke<Device>("unarchive_device", { id });
  },
};
