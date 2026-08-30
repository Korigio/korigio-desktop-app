import type { Session, StaffRole } from "@/features/staff/types/staff";

export type Team = {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  thisDeviceId: string;
  thisDeviceCode: string;
  memberDeviceCount: number;
};

export type CreateTeamInput = {
  name: string;
  memberName: string;
};

export type CreateTeamResult = {
  team: Team;
  pin: string;
  session: Session;
};

export type JoinTeamInput = {
  teamId: string;
  pin: string;
  memberName: string;
};

export type JoinTeamResult = {
  team: Team;
  deviceCode: string;
  snapshotApplied: boolean;
  session: Session;
};

export type TeamPinResult = {
  pin: string;
};

export type NearbyTeam = {
  teamId: string;
  name: string;
};

export type NearbyTeamsResult = {
  items: NearbyTeam[];
};

export type TeamMember = {
  id: string;
  name: string;
  online: boolean;
  gigCount: number;
  role: StaffRole;
};

export type TeamMembersResult = {
  items: TeamMember[];
};

export type SyncStatus = {
  teamId: string | null;
  state: "idle" | "offline" | "catchingUp" | "synced" | "error" | string;
  lastSyncedAt: string | null;
  peersOnline: number;
  pendingOutgoing: number;
  pendingIncoming: number;
  errorMessage: string | null;
};
