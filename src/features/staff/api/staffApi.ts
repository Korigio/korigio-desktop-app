import { invoke } from "@/shared/api/invoke";
import type {
  Session,
  StaffListQuery,
  StaffListResult,
} from "@/features/staff/types/staff";

export const staffApi = {
  list(query: StaffListQuery = {}): Promise<StaffListResult> {
    return invoke<StaffListResult>("list_staff", { query });
  },
  getCurrentSession(): Promise<Session | null> {
    return invoke<Session | null>("get_current_session");
  },
};
