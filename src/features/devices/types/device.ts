export type Device = {
  id: number;
  customerId: number;
  deviceType: string | null;
  manufacturer: string | null;
  model: string | null;
  serialNumber: string | null;
  accessories: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type DeviceInput = {
  customerId: number;
  deviceType?: string | null;
  manufacturer?: string | null;
  model?: string | null;
  serialNumber?: string | null;
  accessories?: string | null;
  notes?: string | null;
};

export type DeviceListQuery = {
  query?: string;
  customerId?: number;
  includeArchived?: boolean;
  page?: number;
  pageSize?: number;
};

export type DeviceListResult = {
  items: Device[];
  total: number;
  page: number;
  pageSize: number;
};

export function deviceLabel(device: Device): string {
  const parts = [device.manufacturer, device.model, device.serialNumber].filter(
    Boolean,
  );
  if (parts.length > 0) {
    return parts.join(" · ");
  }
  return device.deviceType ?? `#${device.id}`;
}
