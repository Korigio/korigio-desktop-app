export const BASE_INTAKE_STEPS = [
  "customer",
  "device",
  "details",
  "review",
  "done",
] as const;

export type IntakeStep = "company" | (typeof BASE_INTAKE_STEPS)[number];

export type IntakeGate = "loading" | "no-company" | "ready";

export type RepairIntakePresets = {
  presetCustomerId?: string;
  presetDeviceId?: string;
};

export type RepairIntakeReviewItem = {
  label: string;
  value: string;
};
