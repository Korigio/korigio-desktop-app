import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";
import type { Repair } from "@/features/repairs/types/repair";

export type GlobalSearchQuery = {
  query: string;
  limitPerType?: number;
};

export type GlobalSearchResult = {
  customers: Customer[];
  devices: Device[];
  repairs: Repair[];
};

export const EMPTY_SEARCH_RESULT: GlobalSearchResult = {
  customers: [],
  devices: [],
  repairs: [],
};
