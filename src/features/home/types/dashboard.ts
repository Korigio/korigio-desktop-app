export type StatusCount = {
  status: string;
  count: number;
};

export type DashboardRepairRow = {
  id: string;
  repairNumber: string;
  status: string;
  customerName: string;
  customerPhone: string | null;
  updatedAt: string;
  readyAt: string | null;
  daysInStatus: number;
};

export type TodayCounts = {
  received: number;
  collected: number;
};

export type HomeDashboard = {
  statusCounts: StatusCount[];
  readyForPickup: DashboardRepairRow[];
  staleRepairs: DashboardRepairRow[];
  today: TodayCounts;
  staleAfterDays: number;
};
