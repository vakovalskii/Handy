import React, { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { SettingContainer } from "../ui/SettingContainer";
import { PathDisplay } from "../ui/PathDisplay";
import { useSettings } from "../../hooks/useSettings";
import { toast } from "sonner";

interface ModelsDirectorySelectorProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const ModelsDirectorySelector: React.FC<ModelsDirectorySelectorProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const [currentPath, setCurrentPath] = useState<string>("");
  const [loading, setLoading] = useState(true);

  const storedPath = getSetting("custom_models_directory");

  const loadPath = useCallback(async () => {
    try {
      const result = await commands.getModelsDirectory();
      if (result.status === "ok") {
        setCurrentPath(result.data);
      } else {
        toast.error(result.error);
      }
    } catch (err) {
      console.error("Failed to get models directory:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPath();
  }, [loadPath, storedPath]);

  const handlePickFolder = async () => {
    try {
      const result = await commands.pickModelsDirectory();
      if (result.status === "ok" && result.data) {
        const newPath = result.data;
        if (newPath !== currentPath) {
          await updateSetting("custom_models_directory", newPath);
          setCurrentPath(newPath);
          toast.success(t("settings.advanced.modelsDirectory.changed"));
        }
      }
    } catch (err) {
      console.error("Failed to pick models directory:", err);
      toast.error(t("settings.advanced.modelsDirectory.pickError"));
    }
  };

  const handleReset = async () => {
    try {
      await updateSetting("custom_models_directory", null);
      await loadPath();
      toast.success(t("settings.advanced.modelsDirectory.reset"));
    } catch (err) {
      console.error("Failed to reset models directory:", err);
      toast.error(t("settings.advanced.modelsDirectory.resetError"));
    }
  };

  const handleOpen = async () => {
    if (!currentPath) return;
    try {
      await commands.openAppDataDir();
    } catch (openError) {
      console.error("Failed to open directory:", openError);
    }
  };

  if (loading) {
    return (
      <div className="animate-pulse">
        <div className="h-4 bg-gray-200 rounded w-1/3 mb-2"></div>
        <div className="h-8 bg-gray-100 rounded"></div>
      </div>
    );
  }

  return (
    <SettingContainer
      title={t("settings.advanced.modelsDirectory.title")}
      description={t("settings.advanced.modelsDirectory.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="stacked"
    >
      <div className="space-y-2">
        <PathDisplay
          path={currentPath || t("settings.advanced.modelsDirectory.default")}
          onOpen={handleOpen}
          disabled={!currentPath}
        />
        <div className="flex gap-2">
          <button
            onClick={handlePickFolder}
            disabled={isUpdating("custom_models_directory")}
            className="px-3 py-2 rounded-md text-sm font-medium
              bg-[color-mix(in_srgb,var(--color-logo-primary)_15%,transparent)]
              text-[var(--color-logo-primary)]
              hover:bg-[color-mix(in_srgb,var(--color-logo-primary)_25%,transparent)]
              disabled:opacity-40 disabled:cursor-not-allowed
              transition-colors"
          >
            {isUpdating("custom_models_directory")
              ? t("common.loading")
              : t("settings.advanced.modelsDirectory.change")}
          </button>
          {storedPath && (
            <button
              onClick={handleReset}
              disabled={isUpdating("custom_models_directory")}
              className="px-3 py-2 rounded-md text-sm font-medium
                bg-mid-gray/10 text-text/70
                hover:bg-mid-gray/20
                disabled:opacity-40 disabled:cursor-not-allowed
                transition-colors"
            >
              {t("common.reset")}
            </button>
          )}
        </div>
      </div>
    </SettingContainer>
  );
};
