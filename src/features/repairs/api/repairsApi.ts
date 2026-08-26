import { invoke } from "@/shared/api/invoke";
import type {
  Repair,
  RepairInput,
  RepairListQuery,
  RepairListResult,
} from "@/features/repairs/types/repair";

export const repairsApi = {
  list(query: RepairListQuery = {}): Promise<RepairListResult> {
    return invoke<RepairListResult>("list_repairs", { query });
  },
  get(id: number): Promise<Repair> {
    return invoke<Repair>("get_repair", { id });
  },
  create(input: RepairInput): Promise<Repair> {
    return invoke<Repair>("create_repair", { input });
  },
  update(id: number, input: RepairInput): Promise<Repair> {
    return invoke<Repair>("update_repair", { id, input });
  },
};
