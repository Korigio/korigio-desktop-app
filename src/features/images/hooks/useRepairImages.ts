import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import { imagesApi } from "@/features/images/api/imagesApi";
import {
  MAX_IMAGES_PER_REPAIR,
  type RepairImage,
} from "@/features/images/types/image";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useRepairImages(repairId: string) {
  const { t } = useI18n();
  const [images, setImages] = useState<RepairImage[]>([]);
  const [thumbUrls, setThumbUrls] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [lightboxUrl, setLightboxUrl] = useState<string | null>(null);
  const [captionDrafts, setCaptionDrafts] = useState<Record<string, string>>(
    {},
  );
  const imagesRef = useRef<RepairImage[]>([]);
  imagesRef.current = images;

  const reload = useCallback(
    async (reloadOptions?: { silent?: boolean }) => {
      const silent = reloadOptions?.silent === true;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const list = await imagesApi.list(repairId);
        setImages(list);
        setCaptionDrafts((prev) => {
          if (!silent) {
            return Object.fromEntries(
              list.map((image) => [image.id, image.caption ?? ""]),
            );
          }
          const next: Record<string, string> = {};
          const previous = imagesRef.current;
          for (const image of list) {
            const prevSaved =
              previous.find((item) => item.id === image.id)?.caption ?? "";
            const draft = prev[image.id];
            next[image.id] =
              draft !== undefined && draft !== prevSaved
                ? draft
                : (image.caption ?? "");
          }
          return next;
        });

        const urls: Record<string, string> = {};
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
        setError(null);
      } catch (err) {
        if (!silent) {
          setError(
            err instanceof Error ? err.message : t("images.errors.loadFailed"),
          );
          setImages([]);
          setThumbUrls({});
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [repairId, t],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

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
    async (imageId: string) => {
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

  const setCaptionDraft = useCallback((imageId: string, value: string) => {
    setCaptionDrafts((prev) => ({ ...prev, [imageId]: value }));
  }, []);

  const saveCaption = useCallback(
    async (imageId: string) => {
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
    async (imageId: string) => {
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
