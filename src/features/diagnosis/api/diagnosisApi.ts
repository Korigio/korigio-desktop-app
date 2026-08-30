import { invoke } from "@/shared/api/invoke";
import type {
  DiagnosisTemplate,
  DiagnosisTemplateInput,
  DiagnosisTemplateListQuery,
  DiagnosisTemplateListResult,
  RepairDiagnosis,
  RepairDiagnosisInput,
} from "@/features/diagnosis/types/diagnosis";

export const diagnosisTemplatesApi = {
  list(
    query: DiagnosisTemplateListQuery = {},
  ): Promise<DiagnosisTemplateListResult> {
    return invoke<DiagnosisTemplateListResult>("list_diagnosis_templates", {
      query,
    });
  },
  get(id: string): Promise<DiagnosisTemplate> {
    return invoke<DiagnosisTemplate>("get_diagnosis_template", { id });
  },
  create(input: DiagnosisTemplateInput): Promise<DiagnosisTemplate> {
    return invoke<DiagnosisTemplate>("create_diagnosis_template", { input });
  },
  update(
    id: string,
    input: DiagnosisTemplateInput,
  ): Promise<DiagnosisTemplate> {
    return invoke<DiagnosisTemplate>("update_diagnosis_template", {
      id,
      input,
    });
  },
  delete(id: string): Promise<void> {
    return invoke<void>("delete_diagnosis_template", { id });
  },
};

export const repairDiagnosisApi = {
  get(repairId: string): Promise<RepairDiagnosis | null> {
    return invoke<RepairDiagnosis | null>("get_repair_diagnosis", {
      repairId,
    });
  },
  upsert(input: RepairDiagnosisInput): Promise<RepairDiagnosis> {
    return invoke<RepairDiagnosis>("upsert_repair_diagnosis", { input });
  },
};
