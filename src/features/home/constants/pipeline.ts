export const HOME_PIPELINE_STATUSES = [
  "received",
  "diagnosis",
  "waiting_customer",
  "waiting_part",
  "in_repair",
  "ready",
  "awaiting_pickup",
] as const;

export type HomePipelineStatus = (typeof HOME_PIPELINE_STATUSES)[number];

export const HOME_PIPELINE_STATUS_COLOR_VAR: Record<
  HomePipelineStatus,
  string
> = {
  received: "--color-status-received",
  diagnosis: "--color-status-diagnosis",
  waiting_customer: "--color-status-waiting-customer",
  waiting_part: "--color-status-waiting-part",
  in_repair: "--color-status-in-repair",
  ready: "--color-status-ready",
  awaiting_pickup: "--color-status-awaiting-pickup",
};

export const HOME_PIPELINE_STATUS_ACCENT_CLASS: Record<
  HomePipelineStatus,
  string
> = {
  received: "border-l-status-received",
  diagnosis: "border-l-status-diagnosis",
  waiting_customer: "border-l-status-waiting-customer",
  waiting_part: "border-l-status-waiting-part",
  in_repair: "border-l-status-in-repair",
  ready: "border-l-status-ready",
  awaiting_pickup: "border-l-status-awaiting-pickup",
};
