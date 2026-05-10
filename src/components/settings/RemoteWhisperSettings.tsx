import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Select } from "../ui/Select";
import { useSettings } from "../../hooks/useSettings";

interface RemoteWhisperSettingsProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

interface TextSettingFieldProps {
  title: string;
  description: string;
  value: string;
  placeholder?: string;
  disabled?: boolean;
  type?: "text" | "password";
  onCommit: (value: string) => void;
}

const TextSettingField: React.FC<TextSettingFieldProps> = ({
  title,
  description,
  value,
  placeholder,
  disabled = false,
  type = "text",
  onCommit,
}) => {
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  return (
    <SettingContainer
      title={title}
      description={description}
      descriptionMode="tooltip"
      grouped
      layout="stacked"
      disabled={disabled}
    >
      <Input
        type={type}
        value={localValue}
        onChange={(event) => setLocalValue(event.target.value)}
        onBlur={() => onCommit(localValue)}
        placeholder={placeholder}
        variant="compact"
        disabled={disabled}
        className="w-full"
      />
    </SettingContainer>
  );
};

export const RemoteWhisperSettings: React.FC<RemoteWhisperSettingsProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const enabled = getSetting("remote_whisper_enabled") ?? false;
    const baseUrl = getSetting("remote_whisper_base_url") ?? "";
    const apiKey = getSetting("remote_whisper_api_key") ?? "";
    const model = getSetting("remote_whisper_model") ?? "whisper-1";
    const prompt = getSetting("remote_whisper_prompt") ?? "";
    const language = getSetting("remote_whisper_language") ?? "auto";
    const temperature = getSetting("remote_whisper_temperature") ?? 0;

    const updateIfChanged = (key: any, value: any, current: any) => {
      if (value !== current) {
        void updateSetting(key, value);
      }
    };

    const determineProvider = (url: string) => {
      if (url.includes("api.groq.com")) return "groq";
      if (url.includes("api.openai.com")) return "openai";
      return "custom";
    };

    const provider = determineProvider(baseUrl);

    const providerOptions = [
      { value: "custom", label: "Custom" },
      { value: "openai", label: "OpenAI" },
      { value: "groq", label: "Groq" },
    ];

    const handleProviderChange = (newProvider: string | null) => {
      if (!newProvider) return;
      if (newProvider === "groq") {
        updateSetting("remote_whisper_base_url", "https://api.groq.com/openai/v1");
        updateSetting("remote_whisper_model", "whisper-large-v3-turbo");
      } else if (newProvider === "openai") {
        updateSetting("remote_whisper_base_url", "https://api.openai.com/v1");
        updateSetting("remote_whisper_model", "whisper-1");
      }
    };

    const isPresetProvider = provider !== "custom";

    return (
      <div className={`${grouped ? "" : "rounded-lg border border-mid-gray/20"}`}>
        <ToggleSwitch
          checked={enabled}
          onChange={(next) => updateSetting("remote_whisper_enabled", next)}
          isUpdating={isUpdating("remote_whisper_enabled")}
          label={t("settings.advanced.remoteWhisper.toggle.label")}
          description={t("settings.advanced.remoteWhisper.toggle.description")}
          descriptionMode={descriptionMode}
          grouped
          tooltipPosition="bottom"
        />
        {enabled && (
          <div className="px-4 pb-2 space-y-2">
            <SettingContainer
              title="Provider"
              description="Select the API provider for remote whisper."
              descriptionMode="tooltip"
              grouped
              layout="stacked"
            >
              <Select
                value={provider}
                options={providerOptions}
                onChange={handleProviderChange}
                isClearable={false}
              />
            </SettingContainer>
            
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.baseUrl.title")}
              description={t("settings.advanced.remoteWhisper.baseUrl.description")}
              value={baseUrl}
              placeholder="https://whisper.example.com/v1"
              disabled={isUpdating("remote_whisper_base_url") || isPresetProvider}
              onCommit={(value) =>
                updateIfChanged(
                  "remote_whisper_base_url",
                  value.trim(),
                  baseUrl,
                )
              }
            />
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.apiKey.title")}
              description={t("settings.advanced.remoteWhisper.apiKey.description")}
              value={apiKey}
              placeholder="your-token"
              type="password"
              disabled={isUpdating("remote_whisper_api_key")}
              onCommit={(value) =>
                updateIfChanged(
                  "remote_whisper_api_key",
                  value.trim(),
                  apiKey,
                )
              }
            />
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.model.title")}
              description={t("settings.advanced.remoteWhisper.model.description")}
              value={model}
              placeholder="whisper-1"
              disabled={isUpdating("remote_whisper_model")}
              onCommit={(value) =>
                updateIfChanged("remote_whisper_model", value.trim(), model)
              }
            />
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.language.title")}
              description={t("settings.advanced.remoteWhisper.language.description")}
              value={language}
              placeholder="auto"
              disabled={isUpdating("remote_whisper_language")}
              onCommit={(value) =>
                updateIfChanged(
                  "remote_whisper_language",
                  value.trim() || "auto",
                  language,
                )
              }
            />
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.prompt.title")}
              description={t("settings.advanced.remoteWhisper.prompt.description")}
              value={prompt}
              placeholder=""
              disabled={isUpdating("remote_whisper_prompt")}
              onCommit={(value) =>
                updateIfChanged("remote_whisper_prompt", value.trim(), prompt)
              }
            />
            <TextSettingField
              title={t("settings.advanced.remoteWhisper.temperature.title")}
              description={t(
                "settings.advanced.remoteWhisper.temperature.description",
              )}
              value={String(temperature)}
              placeholder="0"
              disabled={isUpdating("remote_whisper_temperature")}
              onCommit={(value) => {
                const parsed = Number(value);
                if (!Number.isFinite(parsed)) {
                  return;
                }
                const clamped = Math.max(0, Math.min(2, parsed));
                updateIfChanged(
                  "remote_whisper_temperature",
                  clamped,
                  temperature,
                );
              }}
            />
          </div>
        )}
      </div>
    );
  });
