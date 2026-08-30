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

export type IntakeDayCounts = {
  date: string;
  received: number;
  collected: number;
};

export type RevenueTotals = {
  collectedGrossCentsToday: number;
  collectedGrossCentsWeek: number;
  openEstimateGrossCents: number;
};

export type RevenueDayTotals = {
  date: string;
  collectedGrossCents: number;
};

export type HomeDashboard = {
  statusCounts: StatusCount[];
  readyForPickup: DashboardRepairRow[];
  staleRepairs: DashboardRepairRow[];
  today: TodayCounts;
  intakeByDay: IntakeDayCounts[];
  staleCount: number;
  staleAfterDays: number;
  currency: string;
  revenue: RevenueTotals | null;
  revenueByDay: RevenueDayTotals[] | null;
};
