import { useTranslation } from "react-i18next";
import { Link2 } from "lucide-react";

interface ClaudeThinkingMappingProps {
    value?: boolean;
    onChange: (value: boolean) => void;
}

export default function ClaudeThinkingMapping({
    value = true,
    onChange,
}: ClaudeThinkingMappingProps) {
    const { t } = useTranslation();

    return (
        <div className="space-y-3">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-amber-50/40 dark:bg-amber-900/10 border border-amber-100/60 dark:border-amber-800/20 rounded-lg px-4 py-3">
                <div className="flex items-center gap-3">
                    <div className="p-1.5 bg-amber-100 dark:bg-amber-900/30 rounded text-amber-600 dark:text-amber-400">
                        <Link2 size={18} />
                    </div>
                    <div className="space-y-0.5">
                        <h4 className="font-bold text-sm text-gray-900 dark:text-gray-100">
                            {t("settings.claude_thinking_mapping.title", { defaultValue: "Claude Thinking Model Mapping" })}
                        </h4>
                        <p className="text-[10px] text-gray-500 dark:text-gray-400">
                            {t("settings.claude_thinking_mapping.hint", { defaultValue: "Map thinking suffixed models to upstream base ids." })}
                        </p>
                    </div>
                </div>

                <label className="relative inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        className="sr-only peer"
                        checked={value}
                        onChange={(e) => onChange(e.target.checked)}
                    />
                    <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-amber-500 shadow-inner"></div>
                </label>
            </div>

            <div className="px-1">
                <p className="text-[10px] text-gray-400 dark:text-gray-500 italic leading-relaxed">
                    {value
                        ? t("settings.claude_thinking_mapping.enabled_desc", {
                            defaultValue: "Enabled: claude-*-thinking maps to the base model while keeping thinking params.",
                        })
                        : t("settings.claude_thinking_mapping.disabled_desc", {
                            defaultValue: "Disabled: keep original thinking model ids and rely on upstream defaults.",
                        })}
                </p>
            </div>
        </div>
    );
}
