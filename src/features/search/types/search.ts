import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";
import type { RepairListItem } from "@/features/repairs/types/repair";

export type GlobalSearchQuery = {
  query: string;
  limitPerType?: number;
};

export type GlobalSearchResult = {
  customers: Customer[];
  devices: Device[];
  repairs: RepairListItem[];
};

export const EMPTY_SEARCH_RESULT: GlobalSearchResult = {
  customers: [],
  devices: [],
  repairs: [],
};
