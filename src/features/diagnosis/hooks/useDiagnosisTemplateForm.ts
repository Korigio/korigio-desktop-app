import { useForm } from "@tanstack/react-form";
import { useNavigate } from "react-router";
import { useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import {
  newTemplateItemId,
  type DiagnosisItemKind,
  type DiagnosisTemplate,
  type DiagnosisTemplateInput,
} from "@/features/diagnosis/types/diagnosis";

export type DiagnosisTemplateFormValues = {
  name: string;
  items: Array<{
    id: string;
    label: string;
    kind: DiagnosisItemKind;
  }>;
};

type Options = {
  template?: DiagnosisTemplate;
  mode: "create" | "edit";
};

function toValues(template?: DiagnosisTemplate): DiagnosisTemplateFormValues {
  if (template && template.body.items.length > 0) {
    return {
      name: template.name,
      items: template.body.items.map((item) => ({
        id: item.id,
        label: item.label,
        kind: item.kind,
      })),
    };
  }
  return {
    name: "",
    items: [{ id: newTemplateItemId(), label: "", kind: "checkbox" }],
  };
}

export function useDiagnosisTemplateForm({ template, mode }: Options) {
  const navigate = useNavigate();
  const [submitError, setSubmitError] = useState<string | null>(null);

  const form = useForm({
    defaultValues: toValues(template),
    onSubmit: async ({ value }) => {
      setSubmitError(null);
      const input: DiagnosisTemplateInput = {
        name: value.name.trim(),
        body: {
          items: value.items.map((item) => ({
            id: item.id.trim() || newTemplateItemId(),
            label: item.label.trim(),
            kind: item.kind,
          })),
        },
      };

      try {
        if (mode === "create") {
          await diagnosisTemplatesApi.create(input);
          navigate("/diagnosis-templates");
          return;
        }

        if (!template) {
          throw new Error("Missing template for edit");
        }
        await diagnosisTemplatesApi.update(template.id, input);
        navigate("/diagnosis-templates");
      } catch (err) {
        setSubmitError(
          err instanceof Error ? err.message : "Failed to save template",
        );
        throw err;
      }
    },
  });

  return { form, submitError };
}
