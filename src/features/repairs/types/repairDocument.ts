export const REPAIR_DOCUMENT_TYPES = [
  "entranceSigned",
  "diagnosisSigned",
  "summarySigned",
] as const;

export type RepairDocumentType = (typeof REPAIR_DOCUMENT_TYPES)[number];

export type RepairDocument = {
  repairId: number;
  documentType: RepairDocumentType;
  originalFilename: string;
  createdAt: string;
  updatedAt: string;
};

export const REPAIR_DOCUMENT_PRINT_PATHS: Record<
  RepairDocumentType,
  (repairId: number) => string
> = {
  entranceSigned: (repairId) => `/repairs/${repairId}/print`,
  diagnosisSigned: (repairId) => `/repairs/${repairId}/print/diagnosis`,
  summarySigned: (repairId) => `/repairs/${repairId}/print/summary`,
};
