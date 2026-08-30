export type StaffRole = "admin" | "staff";

export type Staff = {
  id: string;
  teamId: string | null;
  name: string;
  role: StaffRole;
  deactivatedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type Session = {
  staff: Staff;
  deviceId: string;
  deviceName: string;
  deviceCode: string | null;
  startedAt: string;
};

export type StaffListQuery = {
  query?: string;
  includeDeactivated?: boolean;
};

export type StaffListResult = {
  items: Staff[];
  total: number;
};
