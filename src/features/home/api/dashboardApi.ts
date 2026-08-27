import { invoke } from "@/shared/api/invoke";
import type { HomeDashboard } from "@/features/home/types/dashboard";

export const dashboardApi = {
  get(): Promise<HomeDashboard> {
    return invoke<HomeDashboard>("get_home_dashboard");
  },
};
