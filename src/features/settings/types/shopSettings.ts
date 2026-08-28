export type ShopSettings = {
  taxRatePercent: string;
  currency: string;
};

export type ShopSettingsInput = {
  taxRatePercent?: string | null;
  currency?: string | null;
};
