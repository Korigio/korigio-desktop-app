import { invoke } from "@/shared/api/invoke";

export type SeedSyntheticDataInput = {
  customers?: number;
  devices?: number;
  repairs?: number;
  confirm: boolean;
};

export type SeedSyntheticDataResult = {
  customers: number;
  devices: number;
  repairs: number;
  elapsedMs: number;
};

export const seedApi = {
  run(input: SeedSyntheticDataInput): Promise<SeedSyntheticDataResult> {
    return invoke<SeedSyntheticDataResult>("seed_synthetic_data", { input });
  },
};
