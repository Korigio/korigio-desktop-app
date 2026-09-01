export type AppVersion = {
  version: string;
};

export type UpdateCheck = {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  downloadUrl: string | null;
};
