import { invoke } from "@/shared/api/invoke";
import type {
  AppVersion,
  UpdateCheck,
} from "@/features/settings/types/updates";

export const updatesApi = {
  getAppVersion(): Promise<AppVersion> {
    return invoke<AppVersion>("get_app_version");
  },
  checkAppUpdate(): Promise<UpdateCheck> {
    return invoke<UpdateCheck>("check_app_update");
  },
  openExternalUrl(url: string): Promise<void> {
    return invoke<void>("open_external_url", { url });
  },
};
