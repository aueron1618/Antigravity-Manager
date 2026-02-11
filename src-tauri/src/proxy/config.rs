use serde::{Deserialize, Serialize};
use url::Url;
// use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

// ============================================================================
// 辅助工具函数
// ============================================================================

/// 标准化代理 URL，如果缺失协议则默认补全 http://
pub fn normalize_proxy_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url.to_string()
    }
}

pub fn resolve_endpoint_proxy_host_header() -> Option<String> {
    let cfg = get_endpoint_proxy_config();
    if !cfg.enabled {
        return None;
    }

    let raw = cfg.host.as_ref()?.trim();
    if raw.is_empty() {
        return None;
    }

    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{}", raw)
    };

    if let Ok(url) = Url::parse(&candidate) {
        if let Some(host) = url.host_str() {
            if let Some(port) = url.port() {
                return Some(format!("{}:{}", host, port));
            }
            return Some(host.to_string());
        }
    }

    Some(raw.to_string())
}

// ============================================================================
// 全局 Thinking Budget 配置存储
// 用于在 request transform 函数中访问配置（无需修改函数签名）
// ============================================================================
static GLOBAL_THINKING_BUDGET_CONFIG: OnceLock<RwLock<ThinkingBudgetConfig>> = OnceLock::new();

// ============================================================================
// Claude Thinking Mapping 配置存储
// 控制是否将 claude-*-thinking 映射为上游非思考模型
// ============================================================================
static GLOBAL_CLAUDE_THINKING_MAPPING: OnceLock<RwLock<bool>> = OnceLock::new();

/// 获取当前 Thinking Budget 配置
pub fn get_thinking_budget_config() -> ThinkingBudgetConfig {
    GLOBAL_THINKING_BUDGET_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 更新全局 Thinking Budget 配置
pub fn update_thinking_budget_config(config: ThinkingBudgetConfig) {
    if let Some(lock) = GLOBAL_THINKING_BUDGET_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Thinking-Budget] Global config updated: mode={:?}, custom_value={}",
                config.mode,
                config.custom_value
            );
        }
    } else {
        // 首次初始化
        let _ = GLOBAL_THINKING_BUDGET_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Thinking-Budget] Global config initialized: mode={:?}, custom_value={}",
            config.mode,
            config.custom_value
        );
    }
}

/// 获取 Claude Thinking Mapping 是否启用
pub fn get_claude_thinking_mapping_enabled() -> bool {
    GLOBAL_CLAUDE_THINKING_MAPPING
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|val| *val)
        .unwrap_or(true)
}

/// 更新 Claude Thinking Mapping 配置
pub fn update_claude_thinking_mapping_enabled(enabled: bool) {
    if let Some(lock) = GLOBAL_CLAUDE_THINKING_MAPPING.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = enabled;
            tracing::info!(
                "[Claude-Thinking-Mapping] Global config updated: enabled={}",
                enabled
            );
        }
    } else {
        let _ = GLOBAL_CLAUDE_THINKING_MAPPING.set(RwLock::new(enabled));
        tracing::info!(
            "[Claude-Thinking-Mapping] Global config initialized: enabled={}",
            enabled
        );
    }
}

// ============================================================================
// 全局系统提示词配置存储
// 用户可在设置中配置一段全局提示词，自动注入到所有请求的 systemInstruction 中
// ============================================================================
static GLOBAL_SYSTEM_PROMPT_CONFIG: OnceLock<RwLock<GlobalSystemPromptConfig>> = OnceLock::new();

// ============================================================================
// Antigravity 身份指令配置存储
// 允许用户自定义注入到所有请求的 Antigravity 身份指令内容
// ============================================================================
static GLOBAL_ANTIGRAVITY_IDENTITY_CONFIG: OnceLock<RwLock<AntigravityIdentityConfig>> =
    OnceLock::new();

/// 获取当前 Antigravity 身份指令配置
pub fn get_antigravity_identity_config() -> AntigravityIdentityConfig {
    GLOBAL_ANTIGRAVITY_IDENTITY_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 获取当前 Antigravity 身份指令内容（空内容时返回默认值）
pub fn get_antigravity_identity_content() -> Option<String> {
    let config = get_antigravity_identity_config();
    if !config.enabled {
        return None;
    }

    if config.content.trim().is_empty() {
        return Some(default_antigravity_identity_content());
    }

    Some(config.content)
}

/// 更新 Antigravity 身份指令配置
pub fn update_antigravity_identity_config(config: AntigravityIdentityConfig) {
    if let Some(lock) = GLOBAL_ANTIGRAVITY_IDENTITY_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Antigravity-Identity] Config updated: enabled={}, content_len={}",
                config.enabled,
                config.content.len()
            );
        }
    } else {
        let _ = GLOBAL_ANTIGRAVITY_IDENTITY_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Antigravity-Identity] Config initialized: enabled={}, content_len={}",
            config.enabled,
            config.content.len()
        );
    }
}

/// 获取当前全局系统提示词配置
pub fn get_global_system_prompt() -> GlobalSystemPromptConfig {
    GLOBAL_SYSTEM_PROMPT_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 更新全局系统提示词配置
pub fn update_global_system_prompt_config(config: GlobalSystemPromptConfig) {
    if let Some(lock) = GLOBAL_SYSTEM_PROMPT_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Global-System-Prompt] Config updated: enabled={}, content_len={}",
                config.enabled,
                config.content.len()
            );
        }
    } else {
        // 首次初始化
        let _ = GLOBAL_SYSTEM_PROMPT_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Global-System-Prompt] Config initialized: enabled={}, content_len={}",
            config.enabled,
            config.content.len()
        );
    }
}

// ============================================================================
// 全局图像思维模式配置存储
// ============================================================================
static GLOBAL_IMAGE_THINKING_MODE: OnceLock<RwLock<String>> = OnceLock::new();

pub fn get_image_thinking_mode() -> String {
    GLOBAL_IMAGE_THINKING_MODE
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|s| s.clone())
        .unwrap_or_else(|| "enabled".to_string())
}

pub fn update_image_thinking_mode(mode: Option<String>) {
    let val = mode.unwrap_or_else(|| "enabled".to_string());
    if let Some(lock) = GLOBAL_IMAGE_THINKING_MODE.get() {
        if let Ok(mut cfg) = lock.write() {
            if *cfg != val {
                *cfg = val.clone();
                tracing::info!("[Image-Thinking] Global config updated: {}", val);
            }
        }
    } else {
        let _ = GLOBAL_IMAGE_THINKING_MODE.set(RwLock::new(val.clone()));
        tracing::info!("[Image-Thinking] Global config initialized: {}", val);
    }
}

// ============================================================================
// 流式行为配置存储
// ============================================================================
static GLOBAL_STREAM_HANDLING_CONFIG: OnceLock<RwLock<StreamHandlingConfig>> = OnceLock::new();

pub fn get_stream_handling_config() -> StreamHandlingConfig {
    GLOBAL_STREAM_HANDLING_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

pub fn update_stream_handling_config(config: StreamHandlingConfig) {
    if let Some(lock) = GLOBAL_STREAM_HANDLING_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Stream-Handling] Global config updated: fake_non_stream={}, enable_fake_streaming={}",
                config.fake_non_stream,
                config.enable_fake_streaming
            );
        }
    } else {
        let _ = GLOBAL_STREAM_HANDLING_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Stream-Handling] Global config initialized: fake_non_stream={}, enable_fake_streaming={}",
            config.fake_non_stream,
            config.enable_fake_streaming
        );
    }
}

// ============================================================================
// 标点规范化配置存储
// ============================================================================
static GLOBAL_PUNCTUATION_CONFIG: OnceLock<RwLock<PunctuationConfig>> = OnceLock::new();

// ============================================================================
// 上游端点代理配置存储
// ============================================================================
static GLOBAL_ENDPOINT_PROXY_CONFIG: OnceLock<RwLock<EndpointProxyConfig>> = OnceLock::new();

pub fn get_punctuation_config() -> PunctuationConfig {
    GLOBAL_PUNCTUATION_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

pub fn update_punctuation_config(config: PunctuationConfig) {
    if let Some(lock) = GLOBAL_PUNCTUATION_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Punctuation] Global config updated: normalize={}, exclude_tags={}",
                config.normalize,
                config.exclude_tags
            );
        }
    } else {
        let _ = GLOBAL_PUNCTUATION_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Punctuation] Global config initialized: normalize={}, exclude_tags={}",
            config.normalize,
            config.exclude_tags
        );
    }
}

/// 获取上游端点代理配置
pub fn get_endpoint_proxy_config() -> EndpointProxyConfig {
    GLOBAL_ENDPOINT_PROXY_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 更新上游端点代理配置
pub fn update_endpoint_proxy_config(config: EndpointProxyConfig) {
    if let Some(lock) = GLOBAL_ENDPOINT_PROXY_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Endpoint-Proxy] Global config updated: enabled={}, base_urls={} load_code_assist={} oauth_url={} googleapis_url={} host={}",
                config.enabled,
                config.base_urls.len(),
                config
                    .load_code_assist_url
                    .as_ref()
                    .map(|s| s.len())
                    .unwrap_or(0),
                config.oauth_url.as_ref().map(|s| s.len()).unwrap_or(0),
                config
                    .googleapis_url
                    .as_ref()
                    .map(|s| s.len())
                    .unwrap_or(0),
                config.host.as_ref().map(|s| s.len()).unwrap_or(0)
            );
        }
    } else {
        let _ = GLOBAL_ENDPOINT_PROXY_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Endpoint-Proxy] Global config initialized: enabled={}, base_urls={} load_code_assist={} oauth_url={} googleapis_url={} host={}",
            config.enabled,
            config.base_urls.len(),
            config
                .load_code_assist_url
                .as_ref()
                .map(|s| s.len())
                .unwrap_or(0),
            config.oauth_url.as_ref().map(|s| s.len()).unwrap_or(0),
            config
                .googleapis_url
                .as_ref()
                .map(|s| s.len())
                .unwrap_or(0),
            config.host.as_ref().map(|s| s.len()).unwrap_or(0)
        );
    }
}

const DEFAULT_ANTIGRAVITY_IDENTITY_CONTENT: &str =
    "You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.\nYou are pair programming with a USER to solve their coding task. The task may require creating a new codebase, modifying or debugging an existing codebase, or simply answering a question.\n**Absolute paths only**\n**Proactiveness**";

fn default_antigravity_identity_content() -> String {
    DEFAULT_ANTIGRAVITY_IDENTITY_CONTENT.to_string()
}

/// Antigravity 身份指令配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityIdentityConfig {
    /// 是否启用身份指令注入
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 身份指令内容
    #[serde(default = "default_antigravity_identity_content")]
    pub content: String,
}

impl Default for AntigravityIdentityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            content: default_antigravity_identity_content(),
        }
    }
}

/// 全局系统提示词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSystemPromptConfig {
    /// 是否启用全局系统提示词
    #[serde(default)]
    pub enabled: bool,
    /// 系统提示词内容
    #[serde(default)]
    pub content: String,
}

impl Default for GlobalSystemPromptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            content: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyAuthMode {
    Off,
    Strict,
    AllExceptHealth,
    Auto,
}

impl Default for ProxyAuthMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ZaiDispatchMode {
    /// Never use z.ai.
    Off,
    /// Use z.ai for all Anthropic protocol requests.
    Exclusive,
    /// Treat z.ai as one additional slot in the shared pool.
    Pooled,
    /// Use z.ai only when the Google pool is unavailable.
    Fallback,
}

impl Default for ZaiDispatchMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiModelDefaults {
    /// Default model for "opus" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_opus_model")]
    pub opus: String,
    /// Default model for "sonnet" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_sonnet_model")]
    pub sonnet: String,
    /// Default model for "haiku" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_haiku_model")]
    pub haiku: String,
}

impl Default for ZaiModelDefaults {
    fn default() -> Self {
        Self {
            opus: default_zai_opus_model(),
            sonnet: default_zai_sonnet_model(),
            haiku: default_zai_haiku_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiMcpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub web_search_enabled: bool,
    #[serde(default)]
    pub web_reader_enabled: bool,
    #[serde(default)]
    pub vision_enabled: bool,
}

impl Default for ZaiMcpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            web_search_enabled: false,
            web_reader_enabled: false,
            vision_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_zai_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub dispatch_mode: ZaiDispatchMode,
    /// Optional per-model mapping overrides for Anthropic/Claude model ids.
    /// Key: incoming `model` string, Value: upstream z.ai model id (e.g. `glm-4.7`).
    #[serde(default)]
    pub model_mapping: HashMap<String, String>,
    #[serde(default)]
    pub models: ZaiModelDefaults,
    #[serde(default)]
    pub mcp: ZaiMcpConfig,
}

impl Default for ZaiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: default_zai_base_url(),
            api_key: String::new(),
            dispatch_mode: ZaiDispatchMode::Off,
            model_mapping: HashMap::new(),
            models: ZaiModelDefaults::default(),
            mcp: ZaiMcpConfig::default(),
        }
    }
}

/// 实验性功能配置 (Feature Flags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    /// 启用双层签名缓存 (Signature Cache)
    #[serde(default = "default_true")]
    pub enable_signature_cache: bool,

    /// 启用工具循环自动恢复 (Tool Loop Recovery)
    #[serde(default = "default_true")]
    pub enable_tool_loop_recovery: bool,

    /// 启用跨模型兼容性检查 (Cross-Model Checks)
    #[serde(default = "default_true")]
    pub enable_cross_model_checks: bool,

    /// 启用上下文用量缩放 (Context Usage Scaling)
    /// 激进模式: 缩放用量并激活自动压缩以突破 200k 限制
    /// 默认关闭以保持透明度,让客户端能触发原生压缩指令
    #[serde(default = "default_false")]
    pub enable_usage_scaling: bool,

    /// 上下文压缩阈值 L1 (Tool Trimming)
    #[serde(default = "default_threshold_l1")]
    pub context_compression_threshold_l1: f32,

    /// 上下文压缩阈值 L2 (Thinking Compression)
    #[serde(default = "default_threshold_l2")]
    pub context_compression_threshold_l2: f32,

    /// 上下文压缩阈值 L3 (Fork + Summary)
    #[serde(default = "default_threshold_l3")]
    pub context_compression_threshold_l3: f32,
}

impl Default for ExperimentalConfig {
    fn default() -> Self {
        Self {
            enable_signature_cache: true,
            enable_tool_loop_recovery: true,
            enable_cross_model_checks: true,
            enable_usage_scaling: false, // 默认关闭,回归透明模式
            context_compression_threshold_l1: 0.4,
            context_compression_threshold_l2: 0.55,
            context_compression_threshold_l3: 0.7,
        }
    }
}

fn default_threshold_l1() -> f32 {
    0.4
}
fn default_threshold_l2() -> f32 {
    0.55
}
fn default_threshold_l3() -> f32 {
    0.7
}

/// Thinking Budget 模式
/// 控制如何处理调用方传入的 thinking_budget 参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingBudgetMode {
    /// 自动限制：对特定模型（Flash/Thinking）应用 24576 上限
    Auto,
    /// 透传：完全使用调用方传入的值，不做任何修改
    Passthrough,
    /// 自定义：使用用户设定的固定值覆盖所有请求
    Custom,
}

impl Default for ThinkingBudgetMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Thinking Budget 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudgetConfig {
    /// 模式选择
    #[serde(default)]
    pub mode: ThinkingBudgetMode,
    /// 自定义固定值（仅在 mode=Custom 时生效）
    #[serde(default = "default_thinking_budget_custom_value")]
    pub custom_value: u32,
}

impl Default for ThinkingBudgetConfig {
    fn default() -> Self {
        Self {
            mode: ThinkingBudgetMode::Auto,
            custom_value: default_thinking_budget_custom_value(),
        }
    }
}

fn default_thinking_budget_custom_value() -> u32 {
    24576
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_punctuation_exclude_tags() -> String {
    "code,pre,script,style".to_string()
}

fn default_auto_disable_consumption_percent() -> f64 {
    20.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub output_dir: Option<String>,
}

impl Default for DebugLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: None,
        }
    }
}

/// 流式行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHandlingConfig {
    /// 是否启用假非流: 客户端非流式请求将以内流式方式上游请求后再聚合返回
    #[serde(default = "default_true")]
    pub fake_non_stream: bool,
    /// 是否启用假流式模型前缀: 支持 `假流式/` 前缀模型
    #[serde(default = "default_true")]
    pub enable_fake_streaming: bool,
}

impl Default for StreamHandlingConfig {
    fn default() -> Self {
        Self {
            fake_non_stream: true,
            enable_fake_streaming: true,
        }
    }
}

/// 标点规范化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunctuationConfig {
    #[serde(default = "default_false")]
    pub normalize: bool,
    #[serde(default = "default_punctuation_exclude_tags")]
    pub exclude_tags: String,
}

impl Default for PunctuationConfig {
    fn default() -> Self {
        Self {
            normalize: false,
            exclude_tags: default_punctuation_exclude_tags(),
        }
    }
}

/// 上游端点代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointProxyConfig {
    /// 是否启用端点代理
    #[serde(default)]
    pub enabled: bool,
    /// v1internal 基础 URL 列表 (支持多端点降级)
    #[serde(default)]
    pub base_urls: Vec<String>,
    /// loadCodeAssist 端点 (可选)
    #[serde(default)]
    pub load_code_assist_url: Option<String>,
    /// OAuth 端点基础地址 (可选)
    #[serde(default)]
    pub oauth_url: Option<String>,
    /// Google APIs 端点基础地址 (可选)
    #[serde(default)]
    pub googleapis_url: Option<String>,
    /// 上游 Host header 覆盖 (可选)
    #[serde(default)]
    pub host: Option<String>,
}

impl Default for EndpointProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_urls: Vec::new(),
            load_code_assist_url: None,
            oauth_url: None,
            googleapis_url: None,
            host: None,
        }
    }
}

/// IP 黑名单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlacklistConfig {
    /// 是否启用黑名单
    #[serde(default)]
    pub enabled: bool,

    /// 自定义封禁消息
    #[serde(default = "default_block_message")]
    pub block_message: String,
}

impl Default for IpBlacklistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            block_message: default_block_message(),
        }
    }
}

fn default_block_message() -> String {
    "Access denied".to_string()
}

/// IP 白名单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistConfig {
    /// 是否启用白名单模式 (启用后只允许白名单IP访问)
    #[serde(default)]
    pub enabled: bool,

    /// 白名单优先模式 (白名单IP跳过黑名单检查)
    #[serde(default = "default_true")]
    pub whitelist_priority: bool,
}

impl Default for IpWhitelistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            whitelist_priority: true,
        }
    }
}

/// 安全监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitorConfig {
    /// IP 黑名单配置
    #[serde(default)]
    pub blacklist: IpBlacklistConfig,

    /// IP 白名单配置
    #[serde(default)]
    pub whitelist: IpWhitelistConfig,
}

impl Default for SecurityMonitorConfig {
    fn default() -> Self {
        Self {
            blacklist: IpBlacklistConfig::default(),
            whitelist: IpWhitelistConfig::default(),
        }
    }
}

/// 反代服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 是否启用反代服务
    pub enabled: bool,

    /// 是否允许局域网访问
    /// - false: 仅本机访问 127.0.0.1（默认，隐私优先）
    /// - true: 允许局域网访问 0.0.0.0
    #[serde(default)]
    pub allow_lan_access: bool,

    /// Authorization policy for the proxy.
    /// - off: no auth required
    /// - strict: auth required for all routes
    /// - all_except_health: auth required for all routes except `/healthz`
    /// - auto: recommended defaults (currently: allow_lan_access => all_except_health, else off)
    #[serde(default)]
    pub auth_mode: ProxyAuthMode,

    /// 监听端口
    pub port: u16,

    /// API 密钥
    pub api_key: String,

    /// Web UI 管理后台密码 (可选，如未设置则使用 api_key)
    pub admin_password: Option<String>,

    /// 是否自动启动
    pub auto_start: bool,

    /// 自定义精确模型映射表 (key: 原始模型名, value: 目标模型名)
    #[serde(default)]
    pub custom_mapping: std::collections::HashMap<String, String>,

    /// API 请求超时时间(秒)
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// 流式行为配置（假非流/假流式前缀）
    #[serde(default)]
    pub stream_handling: StreamHandlingConfig,

    /// 标点规范化配置
    #[serde(default)]
    pub punctuation: PunctuationConfig,

    /// 是否启用按消耗比例自动禁用账号
    #[serde(default = "default_true")]
    pub auto_disable_on_consumption: bool,

    /// 自动禁用阈值（消耗百分比，1-99）
    /// 当任一模型剩余额度 <= 100 - 此值 时触发自动禁用
    #[serde(default = "default_auto_disable_consumption_percent")]
    pub auto_disable_consumption_percent: f64,

    /// 是否开启请求日志记录 (监控)
    #[serde(default)]
    pub enable_logging: bool,

    /// 调试日志配置 (保存完整链路)
    #[serde(default)]
    pub debug_logging: DebugLoggingConfig,

    /// 上游代理配置
    #[serde(default)]
    pub upstream_proxy: UpstreamProxyConfig,

    /// z.ai provider configuration (Anthropic-compatible).
    #[serde(default)]
    pub zai: ZaiConfig,

    /// 自定义 User-Agent 请求头 (可选覆盖)
    #[serde(default)]
    pub user_agent_override: Option<String>,

    /// 账号调度配置 (粘性会话/限流重试)
    #[serde(default)]
    pub scheduling: crate::proxy::sticky_config::StickySessionConfig,

    /// 实验性功能配置
    #[serde(default)]
    pub experimental: ExperimentalConfig,

    /// 安全监控配置 (IP 黑白名单)
    #[serde(default)]
    pub security_monitor: SecurityMonitorConfig,

    /// 固定账号模式的账号ID (Fixed Account Mode)
    /// - None: 使用轮询模式
    /// - Some(account_id): 固定使用指定账号
    #[serde(default)]
    pub preferred_account_id: Option<String>,

    /// Saved User-Agent string (persisted even when override is disabled)
    #[serde(default)]
    pub saved_user_agent: Option<String>,

    /// Thinking Budget 配置
    /// 控制如何处理 AI 深度思考时的 Token 预算
    #[serde(default)]
    pub thinking_budget: ThinkingBudgetConfig,

    /// Antigravity 身份指令配置
    /// 自动注入到各协议请求的 systemInstruction 中
    #[serde(default)]
    pub antigravity_identity: AntigravityIdentityConfig,

    /// 全局系统提示词配置
    /// 自动注入到所有 API 请求的 systemInstruction 中
    #[serde(default)]
    pub global_system_prompt: GlobalSystemPromptConfig,

    /// 图像思维模式配置
    /// - enabled: 保留思维链 (默认)
    /// - disabled: 移除思维链 (画质优先)
    #[serde(default)]
    pub image_thinking_mode: Option<String>,

    /// Claude thinking 映射开关
    /// - true: claude-*-thinking 映射为非思考模型，同时仍启用 thinking 参数
    /// - false: 透传 thinking 模型名
    #[serde(default = "default_true")]
    pub claude_thinking_mapping: bool,

    /// 代理池配置
    #[serde(default)]
    pub proxy_pool: ProxyPoolConfig,

    /// 上游端点代理配置
    #[serde(default)]
    pub endpoint_proxy: EndpointProxyConfig,
}

/// 上游代理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpstreamProxyConfig {
    /// 是否启用
    pub enabled: bool,
    /// 代理地址 (http://, https://, socks5://)
    pub url: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_lan_access: false, // 默认仅本机访问，隐私优先
            auth_mode: ProxyAuthMode::default(),
            port: 8045,
            api_key: format!("sk-{}", uuid::Uuid::new_v4().simple()),
            admin_password: None,
            auto_start: false,
            custom_mapping: std::collections::HashMap::new(),
            request_timeout: default_request_timeout(),
            stream_handling: StreamHandlingConfig::default(),
            punctuation: PunctuationConfig::default(),
            auto_disable_on_consumption: true,
            auto_disable_consumption_percent: default_auto_disable_consumption_percent(),
            enable_logging: true, // 默认开启，支持 token 统计功能
            debug_logging: DebugLoggingConfig::default(),
            upstream_proxy: UpstreamProxyConfig::default(),
            zai: ZaiConfig::default(),
            scheduling: crate::proxy::sticky_config::StickySessionConfig::default(),
            experimental: ExperimentalConfig::default(),
            security_monitor: SecurityMonitorConfig::default(),
            preferred_account_id: None, // 默认使用轮询模式
            user_agent_override: None,
            saved_user_agent: None,
            thinking_budget: ThinkingBudgetConfig::default(),
            antigravity_identity: AntigravityIdentityConfig::default(),
            global_system_prompt: GlobalSystemPromptConfig::default(),
            proxy_pool: ProxyPoolConfig::default(),
            image_thinking_mode: None,
            claude_thinking_mapping: true,
            endpoint_proxy: EndpointProxyConfig::default(),
        }
    }
}

fn default_request_timeout() -> u64 {
    120 // 默认 120 秒,原来 60 秒太短
}

fn default_zai_base_url() -> String {
    "https://api.z.ai/api/anthropic".to_string()
}

fn default_zai_opus_model() -> String {
    "glm-4.7".to_string()
}

fn default_zai_sonnet_model() -> String {
    "glm-4.7".to_string()
}

fn default_zai_haiku_model() -> String {
    "glm-4.5-air".to_string()
}

impl ProxyConfig {
    /// 获取实际的监听地址
    /// - allow_lan_access = false: 返回 "127.0.0.1"（默认，隐私优先）
    /// - allow_lan_access = true: 返回 "0.0.0.0"（允许局域网访问）
    pub fn get_bind_address(&self) -> &str {
        if self.allow_lan_access {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        }
    }
}

/// 代理认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    #[serde(
        serialize_with = "crate::utils::crypto::serialize_password",
        deserialize_with = "crate::utils::crypto::deserialize_password"
    )]
    pub password: String,
}

/// 单个代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub id: String,                       // 唯一标识
    pub name: String,                     // 显示名称
    pub url: String,                      // 代理地址 (http://, https://, socks5://)
    pub auth: Option<ProxyAuth>,          // 认证信息 (可选)
    pub enabled: bool,                    // 是否启用
    pub priority: i32,                    // 优先级 (数字越小优先级越高)
    pub tags: Vec<String>,                // 标签 (如 "美国", "住宅IP")
    pub max_accounts: Option<usize>,      // 最大绑定账号数 (0 = 无限制)
    pub health_check_url: Option<String>, // 健康检查 URL
    pub last_check_time: Option<i64>,     // 上次检查时间
    pub is_healthy: bool,                 // 健康状态
    pub latency: Option<u64>,             // 延迟 (毫秒) [NEW]
}

/// 代理池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolConfig {
    pub enabled: bool, // 是否启用代理池
    // pub mode: ProxyPoolMode,        // [REMOVED] 代理池模式，统一为 Hybrid 逻辑
    pub proxies: Vec<ProxyEntry>,         // 代理列表
    pub health_check_interval: u64,       // 健康检查间隔 (秒)
    pub auto_failover: bool,              // 自动故障转移
    pub strategy: ProxySelectionStrategy, // 代理选择策略
    /// 账号到代理的绑定关系 (account_id -> proxy_id)，持久化存储
    #[serde(default)]
    pub account_bindings: HashMap<String, String>,
}

impl Default for ProxyPoolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            // mode: ProxyPoolMode::Global,
            proxies: Vec::new(),
            health_check_interval: 300,
            auto_failover: true,
            strategy: ProxySelectionStrategy::Priority,
            account_bindings: HashMap::new(),
        }
    }
}

/// 代理选择策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxySelectionStrategy {
    /// 轮询: 依次使用
    RoundRobin,
    /// 随机: 随机选择
    Random,
    /// 优先级: 按 priority 字段排序
    Priority,
    /// 最少连接: 选择当前使用最少的代理
    LeastConnections,
    /// 加权轮询: 根据健康状态和优先级
    WeightedRoundRobin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_proxy_url() {
        // 测试已有协议
        assert_eq!(
            normalize_proxy_url("http://127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("https://proxy.com"),
            "https://proxy.com"
        );
        assert_eq!(
            normalize_proxy_url("socks5://127.0.0.1:1080"),
            "socks5://127.0.0.1:1080"
        );
        assert_eq!(
            normalize_proxy_url("socks5h://127.0.0.1:1080"),
            "socks5h://127.0.0.1:1080"
        );

        // 测试缺少协议（默认补全 http://）
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("localhost:1082"),
            "http://localhost:1082"
        );

        // 测试边缘情况
        assert_eq!(normalize_proxy_url(""), "");
        assert_eq!(normalize_proxy_url("   "), "");
    }
}
