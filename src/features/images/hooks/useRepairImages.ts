import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { imagesApi } from "@/features/images/api/imagesApi";
import {
  MAX_IMAGES_PER_REPAIR,
  type RepairImage,
} from "@/features/images/types/image";
import { useI18n } from "@/shared/hooks/useI18n";

export function useRepairImages(repairId: number) {
  const { t } = useI18n();
  const [images, setImages] = useState<RepairImage[]>([]);
  const [thumbUrls, setThumbUrls] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [lightboxUrl, setLightboxUrl] = useState<string | null>(null);
  const [captionDrafts, setCaptionDrafts] = useState<Record<number, string>>(
    {},
  );

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await imagesApi.list(repairId);
      setImages(list);
      setCaptionDrafts(
        Object.fromEntries(
          list.map((image) => [image.id, image.caption ?? ""]),
        ),
      );

      const urls: Record<number, string> = {};
      await Promise.all(
        list.map(async (image) => {
          try {
            const variant = image.thumbPath ? "thumb" : "original";
            const resolved = await imagesApi.resolvePath(image.id, variant);
            urls[image.id] = convertFileSrc(resolved.absolutePath);
          } catch {
            // Skip broken paths; row still listed.
          }
        }),
      );
      setThumbUrls(urls);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("images.errors.loadFailed"),
      );
      setImages([]);
      setThumbUrls({});
    } finally {
      setLoading(false);
    }
  }, [repairId, t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const atLimit = images.length >= MAX_IMAGES_PER_REPAIR;

  const addPhotos = useCallback(async () => {
    if (atLimit) {
      setError(t("images.errors.limitReached"));
      return;
    }
    setError(null);
    setSuccess(null);
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Images",
            extensions: ["jpg", "jpeg", "png"],
          },
        ],
      });
      if (selected === null) {
        return;
      }
      const sourcePaths = Array.isArray(selected) ? selected : [selected];
      if (sourcePaths.length === 0) {
        return;
      }

      setBusy(true);
      await imagesApi.attach({ repairId, sourcePaths });
      setSuccess(t("images.addSuccess"));
      await reload();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("images.errors.attachFailed"),
      );
    } finally {
      setBusy(false);
    }
  }, [atLimit, reload, repairId, t]);

  const openLightbox = useCallback(
    async (imageId: number) => {
      setError(null);
      try {
        const resolved = await imagesApi.resolvePath(imageId, "original");
        setLightboxUrl(convertFileSrc(resolved.absolutePath));
      } catch (err) {
        setError(
          err instanceof Error ? err.message : t("images.errors.resolveFailed"),
        );
      }
    },
    [t],
  );

  const closeLightbox = useCallback(() => {
    setLightboxUrl(null);
  }, []);

  const setCaptionDraft = useCallback((imageId: number, value: string) => {
    setCaptionDrafts((prev) => ({ ...prev, [imageId]: value }));
  }, []);

  const saveCaption = useCallback(
    async (imageId: number) => {
      setBusy(true);
      setError(null);
      setSuccess(null);
      try {
        const caption = (captionDrafts[imageId] ?? "").trim();
        await imagesApi.update(imageId, {
          caption: caption.length > 0 ? caption : null,
        });
        setSuccess(t("images.captionSaved"));
        await reload();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : t("images.errors.updateFailed"),
        );
      } finally {
        setBusy(false);
      }
    },
    [captionDrafts, reload, t],
  );

  const removeImage = useCallback(
    async (imageId: number) => {
      const confirmed = window.confirm(t("images.deleteConfirm"));
      if (!confirmed) {
        return;
      }
      setBusy(true);
      setError(null);
      setSuccess(null);
      try {
        await imagesApi.delete(imageId);
        setSuccess(t("images.deleteSuccess"));
        if (lightboxUrl) {
          setLightboxUrl(null);
        }
        await reload();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : t("images.errors.deleteFailed"),
        );
      } finally {
        setBusy(false);
      }
    },
    [lightboxUrl, reload, t],
  );

  return {
    images,
    thumbUrls,
    loading,
    busy,
    error,
    success,
    atLimit,
    lightboxUrl,
    captionDrafts,
    addPhotos,
    openLightbox,
    closeLightbox,
    setCaptionDraft,
    saveCaption,
    removeImage,
  };
}
