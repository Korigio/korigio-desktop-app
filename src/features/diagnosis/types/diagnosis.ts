export type DiagnosisItemKind = "checkbox" | "text";

export type DiagnosisTemplateBodyItem = {
  id: string;
  label: string;
  kind: DiagnosisItemKind;
};

export type DiagnosisTemplateBody = {
  items: DiagnosisTemplateBodyItem[];
};

export type DiagnosisResultItem = {
  id: string;
  label: string;
  kind: DiagnosisItemKind;
  value: boolean | string;
};

export type DiagnosisResult = {
  items: DiagnosisResultItem[];
};

export type DiagnosisTemplate = {
  id: string;
  name: string;
  body: DiagnosisTemplateBody;
  updatedAt: string;
};

export type DiagnosisTemplateInput = {
  name: string;
  body: DiagnosisTemplateBody;
};

export type DiagnosisTemplateListQuery = {
  query?: string;
  page?: number;
  pageSize?: number;
};

export type DiagnosisTemplateListResult = {
  items: DiagnosisTemplate[];
  total: number;
  page: number;
  pageSize: number;
};

export type RepairDiagnosis = {
  id: string;
  repairId: string;
  templateId: string | null;
  result: DiagnosisResult;
};

export type RepairDiagnosisInput = {
  repairId: string;
  templateId?: string | null;
  result: DiagnosisResult;
};

export function defaultResultValue(kind: DiagnosisItemKind): boolean | string {
  return kind === "checkbox" ? false : "";
}

export function templateItemsToResult(
  items: DiagnosisTemplateBodyItem[],
): DiagnosisResultItem[] {
  return items.map((item) => ({
    id: item.id,
    label: item.label,
    kind: item.kind,
    value: defaultResultValue(item.kind),
  }));
}

export function newTemplateItemId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `item-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}
