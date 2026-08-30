import { useRepairImages } from "@/features/images/hooks/useRepairImages";
import {
  Button,
  Card,
  FormField,
  StatusMessage,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repairId: string;
};

export function RepairImagesSection({ repairId }: Props) {
  const { t } = useI18n();
  const state = useRepairImages(repairId);

  return (
    <Card
      title={t("repairs.detail.sections.photos")}
      actions={
        <Button
          type="button"
          disabled={state.busy || state.loading || state.atLimit}
          onClick={() => void state.addPhotos()}
        >
          {state.busy ? t("common.saving") : t("images.actions.add")}
        </Button>
      }
    >
      {state.error ? (
        <StatusMessage tone="danger">{state.error}</StatusMessage>
      ) : null}
      {state.success ? (
        <StatusMessage tone="success">{state.success}</StatusMessage>
      ) : null}

      {state.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : state.images.length === 0 ? (
        <p className="text-sm text-muted">{t("images.empty")}</p>
      ) : (
        <ul className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {state.images.map((image) => (
            <li key={image.id} className="flex flex-col gap-2">
              <button
                type="button"
                className="overflow-hidden rounded-md border border-border bg-surface text-left"
                onClick={() => void state.openLightbox(image.id)}
              >
                {state.thumbUrls[image.id] ? (
                  <img
                    src={state.thumbUrls[image.id]}
                    alt={image.caption ?? t("images.thumbAlt")}
                    loading="lazy"
                    className="aspect-square w-full object-cover"
                  />
                ) : (
                  <div className="flex aspect-square items-center justify-center text-sm text-muted">
                    {t("images.thumbMissing")}
                  </div>
                )}
              </button>
              <FormField
                label={t("images.fields.caption")}
                htmlFor={`image-caption-${image.id}`}
              >
                <TextField
                  id={`image-caption-${image.id}`}
                  value={state.captionDrafts[image.id] ?? ""}
                  disabled={state.busy}
                  onChange={(event) =>
                    state.setCaptionDraft(image.id, event.target.value)
                  }
                />
              </FormField>
              <div className="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="secondary"
                  className="px-2 py-1 text-xs"
                  disabled={state.busy}
                  onClick={() => void state.saveCaption(image.id)}
                >
                  {t("images.actions.saveCaption")}
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  className="px-2 py-1 text-xs"
                  disabled={state.busy}
                  onClick={() => void state.removeImage(image.id)}
                >
                  {t("images.actions.delete")}
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}

      {state.lightboxUrl ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6"
          role="dialog"
          aria-modal="true"
          aria-label={t("images.lightboxTitle")}
          onClick={state.closeLightbox}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              state.closeLightbox();
            }
          }}
        >
          <img
            src={state.lightboxUrl}
            alt={t("images.lightboxTitle")}
            className="max-h-full max-w-full object-contain"
            onClick={(event) => event.stopPropagation()}
          />
        </div>
      ) : null}
    </Card>
  );
}
