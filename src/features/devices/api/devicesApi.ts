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
  get(id: number): Promise<Device> {
    return invoke<Device>("get_device", { id });
  },
  create(input: DeviceInput): Promise<Device> {
    return invoke<Device>("create_device", { input });
  },
  update(id: number, input: DeviceInput): Promise<Device> {
    return invoke<Device>("update_device", { id, input });
  },
  archive(id: number): Promise<Device> {
    return invoke<Device>("archive_device", { id });
  },
  unarchive(id: number): Promise<Device> {
    return invoke<Device>("unarchive_device", { id });
  },
};
