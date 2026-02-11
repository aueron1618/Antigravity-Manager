import { useTranslation } from "react-i18next";
import { AntigravityIdentityConfig } from "../../types/config";

interface AntigravityIdentityProps {
    config?: AntigravityIdentityConfig;
    onChange: (config: AntigravityIdentityConfig) => void;
}

const DEFAULT_CONTENT = "You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.\nYou are pair programming with a USER to solve their coding task. The task may require creating a new codebase, modifying or debugging an existing codebase, or simply answering a question.\n**Absolute paths only**\n**Proactiveness**";

const DEFAULT_CONFIG: AntigravityIdentityConfig = {
    enabled: true,
    content: DEFAULT_CONTENT,
};

export default function AntigravityIdentity({
    config = DEFAULT_CONFIG,
    onChange,
}: AntigravityIdentityProps) {
    const { t } = useTranslation();

    return (
        <div className="space-y-3">
            <div className="flex items-center justify-between gap-3 bg-sky-50/40 dark:bg-sky-900/10 border border-sky-100/60 dark:border-sky-800/30 rounded-lg px-4 py-3">
                <div className="space-y-0.5">
                    <h4 className="font-bold text-sm text-gray-900 dark:text-gray-100">
                        {t("settings.antigravity_identity.title", { defaultValue: "Antigravity 身份指令" })}
                    </h4>
                    <p className="text-[10px] text-gray-500 dark:text-gray-400">
                        {t("settings.antigravity_identity.hint", { defaultValue: "注入到 OpenAI/Gemini/Claude 路由的 systemInstruction" })}
                    </p>
                </div>

                <div className="flex items-center gap-3">
                    <span
                        className={`text-[10px] font-medium ${config.enabled ? 'text-sky-600 dark:text-sky-400' : 'text-gray-400'}`}
                    >
                        {config.enabled ? t("common.enabled", { defaultValue: "已启用" }) : t("common.disabled", { defaultValue: "已禁用" })}
                    </span>
                    <label className="relative inline-flex items-center cursor-pointer shrink-0">
                        <input
                            type="checkbox"
                            checked={config.enabled}
                            onChange={(e) => onChange({ ...config, enabled: e.target.checked })}
                            className="sr-only peer"
                        />
                        <div className="w-9 h-5 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:after:border-gray-600 peer-checked:bg-sky-600"></div>
                    </label>
                </div>
            </div>

            {config.enabled && (
                <div className="space-y-3">
                    <textarea
                        value={config.content}
                        onChange={(e) => onChange({ ...config, content: e.target.value })}
                        placeholder={t("settings.antigravity_identity.placeholder", {
                            defaultValue: DEFAULT_CONTENT,
                        })}
                        rows={6}
                        className="w-full bg-white dark:bg-base-100 border border-gray-200 dark:border-gray-700 rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-sky-500/20 outline-none transition-all resize-y min-h-[120px]"
                    />
                    <div className="flex items-center justify-between">
                        <p className="text-xs text-gray-400 dark:text-gray-500">
                            {t("settings.antigravity_identity.char_count", {
                                defaultValue: "{{count}} 字符",
                                count: config.content.length,
                            })}
                        </p>
                    </div>
                </div>
            )}
        </div>
    );
}
