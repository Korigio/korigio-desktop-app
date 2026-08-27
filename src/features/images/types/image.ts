export type RepairImage = {
  id: number;
  repairId: number;
  originalPath: string;
  thumbPath: string | null;
  caption: string | null;
  sortOrder: number;
  createdAt: string;
};

export type AttachRepairImagesInput = {
  repairId: number;
  sourcePaths: string[];
};

export type UpdateRepairImageInput = {
  caption?: string | null;
  sortOrder?: number;
};

export type ImageVariant = "original" | "thumb";

export type ResolveRepairImagePathResult = {
  absolutePath: string;
};

export const MAX_IMAGES_PER_REPAIR = 30;
