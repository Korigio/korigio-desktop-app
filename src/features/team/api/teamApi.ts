import { invoke } from "@/shared/api/invoke";
import type {
  CreateTeamInput,
  CreateTeamResult,
  JoinTeamInput,
  JoinTeamResult,
  NearbyTeamsResult,
  SyncStatus,
  Team,
  TeamMembersResult,
  TeamPinResult,
} from "@/features/team/types/team";

export const teamApi = {
  get(): Promise<Team | null> {
    return invoke<Team | null>("get_team");
  },
  create(input: CreateTeamInput): Promise<CreateTeamResult> {
    return invoke<CreateTeamResult>("create_team", { input });
  },
  join(input: JoinTeamInput): Promise<JoinTeamResult> {
    return invoke<JoinTeamResult>("join_team", { input });
  },
  getPin(): Promise<TeamPinResult> {
    return invoke<TeamPinResult>("get_team_pin");
  },
  listNearby(): Promise<NearbyTeamsResult> {
    return invoke<NearbyTeamsResult>("list_nearby_teams");
  },
  listMembers(): Promise<TeamMembersResult> {
    return invoke<TeamMembersResult>("list_team_members");
  },
  getSyncStatus(): Promise<SyncStatus> {
    return invoke<SyncStatus>("get_sync_status");
  },
  leave(): Promise<void> {
    return invoke<void>("leave_team");
  },
};
