import { invoke } from "@/shared/api/invoke";
import type {
  AttachRepairImagesInput,
  ImageVariant,
  RepairImage,
  ResolveRepairImagePathResult,
  UpdateRepairImageInput,
} from "@/features/images/types/image";

export const imagesApi = {
  list(repairId: string): Promise<RepairImage[]> {
    return invoke<RepairImage[]>("list_repair_images", { repairId });
  },
  attach(input: AttachRepairImagesInput): Promise<RepairImage[]> {
    return invoke<RepairImage[]>("attach_repair_images", { input });
  },
  update(id: string, input: UpdateRepairImageInput): Promise<RepairImage> {
    return invoke<RepairImage>("update_repair_image", { id, input });
  },
  delete(id: string): Promise<void> {
    return invoke<void>("delete_repair_image", { id });
  },
  resolvePath(
    id: string,
    variant: ImageVariant,
  ): Promise<ResolveRepairImagePathResult> {
    return invoke<ResolveRepairImagePathResult>("resolve_repair_image_path", {
      id,
      variant,
    });
  },
};
