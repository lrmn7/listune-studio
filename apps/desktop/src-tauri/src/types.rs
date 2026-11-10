//! Serde mirrors of the TypeScript types in `packages/shared/src/types.ts`.
//! Field names are camelCase and string unions serialize to the exact TS literals.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentId {
    Claude,
    Opencode,
    Antigravity,
    Codex,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: AgentId,
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub acp_command: Vec<String>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OsKind {
    Macos,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Brew,
    Winget,
    Apt,
    Dnf,
    Pacman,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: OsKind,
    pub os_version: String,
    pub package_manager: Option<PackageManager>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolId {
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "ffmpeg")]
    Ffmpeg,
    #[serde(rename = "chrome")]
    Chrome,
    #[serde(rename = "ytdlp")]
    Ytdlp,
    #[serde(rename = "elevenlabs-mcp")]
    ElevenlabsMcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub id: ToolId,
    pub name: String,
    pub required: bool,
    pub found: bool,
    pub version: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoKind {
    Explainer,
    Promo,
    Quote,
    News,
    Tutorial,
    Data,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoFormat {
    #[serde(rename = "9:16")]
    Portrait,
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "16:9")]
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoLanguage {
    Id,
    En,
}

/// The language of Listune's own interface, not of the videos it makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLanguage {
    Id,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Draft,
    Building,
    Rendered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub folder: String,
    pub kind: VideoKind,
    pub format: VideoFormat,
    pub language: VideoLanguage,
    pub status: ProjectStatus,
    pub style_id: Option<String>,
    pub duration_sec: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProject {
    pub title: String,
    pub kind: VideoKind,
    pub format: VideoFormat,
    pub language: VideoLanguage,
    #[serde(default)]
    pub style_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String,
    pub project_id: Option<String>,
    pub agent_id: AgentId,
    pub title: String,
    pub created_at: String,
}

/// A chat in the sidebar's Recent list: its project's title, and when anything last happened in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentChat {
    pub id: String,
    pub project_id: Option<String>,
    pub agent_id: AgentId,
    pub title: String,
    pub created_at: String,
    pub project_title: Option<String>,
    /// The newest message's time, or the chat's own when it has none (RFC 3339).
    pub active_at: String,
    /// When the thread was archived (RFC 3339); only set in the archive's list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
}

/// Relative position, 0-1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum MessagePart {
    Text {
        text: String,
    },
    Mention {
        project_id: String,
        title: String,
    },
    /// A style the user @mentioned. The app puts its prompt and notes in front of the message.
    #[serde(rename = "style-mention")]
    StyleMention {
        style_id: String,
        name: String,
    },
    File {
        path: String,
        name: String,
        mime: String,
        size: u64,
        /// When the file was last written (an image the agent made): the UI reloads a rewritten one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modified_at: Option<String>,
    },
    Frame {
        project_id: String,
        frame: u32,
        time_sec: f64,
        image_path: String,
        marks: Vec<Mark>,
    },
    Tool {
        tool_call_id: String,
        title: String,
        status: ToolCallStatus,
        /// The ACP tool kind (`read`, `edit`, `execute` ...), when the agent said.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_kind: Option<String>,
        /// The first file the tool touches.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    Thought {
        text: String,
    },
    A2ui {
        surface_id: String,
        messages: Vec<serde_json::Value>,
    },
    #[serde(rename = "a2ui-action")]
    A2uiAction {
        summary: String,
        action: serde_json::Value,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleAspect {
    Camera,
    Pacing,
    Type,
    Color,
    Captions,
    Transitions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleSource {
    Builtin,
    Copied,
    Mine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    pub id: String,
    pub name: String,
    pub source: StyleSource,
    pub source_url: Option<String>,
    /// Partial record: only aspects with notes are present.
    pub notes: BTreeMap<StyleAspect, String>,
    pub prompt: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<StyleEngine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_video_url: Option<String>,
    /// The built-in style this one was cloned from; its template is downloaded under that id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub based_on: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<StyleTemplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    /// Copied styles: how to build the look (engine technique, characters, rig, captions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    /// Copied styles: the main colours as #RRGGBB.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette: Vec<String>,
    /// Copied styles: reference frames on this computer (absolute paths under style-refs/<id>).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleEngine {
    Hyperframes,
    Remotion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleTemplate {
    pub sha256: String,
    pub size: u64,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub installed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobKind {
    Build,
    Render,
    Install,
    StudyStyle,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobStatus {
    Queued,
    Working,
    NeedsYou,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub kind: JobKind,
    pub title: String,
    pub project_id: Option<String>,
    pub chat_id: Option<String>,
    pub agent_id: AgentId,
    pub status: JobStatus,
    pub progress: Option<f64>,
    pub note: Option<String>,
    /// The link a style study was reading, so a failed study can be tried again.
    #[serde(default)]
    pub source_url: Option<String>,
    /// A build job: when the first turn since it was last done started (RFC 3339). A render made
    /// after it is this job's video. None for other jobs, done ones, and jobs saved before it existed.
    #[serde(default)]
    pub render_baseline_at: Option<String>,
    pub updated_at: String,
    /// When the card was archived off the board (RFC 3339). Only the database sets it: a write from the UI
    /// never carries it, and a card whose status changes comes back on the board.
    #[serde(default)]
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub working_folder: String,
    pub default_agent: AgentId,
    pub frame_check: bool,
    pub language: VideoLanguage,
    pub format: VideoFormat,
    pub fps: u32,
    pub onboarded: bool,
    /// Chats start in auto mode: the agent runs commands without asking.
    #[serde(default)]
    pub auto_mode: bool,
    /// Language of the interface. `None` until the first run, when the UI picks one from the
    /// system language and saves it.
    #[serde(default)]
    pub ui_language: Option<UiLanguage>,
    /// Closing the window hides it to the tray instead of quitting.
    #[serde(default)]
    pub background_mode: bool,
    /// Start Listune when the user logs in; only meaningful while `background_mode` is on.
    #[serde(default)]
    pub open_at_login: bool,
    /// The one-time startup check of build jobs marked done without a video has run.
    #[serde(default)]
    pub build_jobs_checked: bool,
    /// The one-time startup backfill of project statuses (rendered, building) has run.
    #[serde(default)]
    pub project_status_checked: bool,
    /// The model picked for each agent (agent id → the agent's own model value). Missing: the
    /// agent's own default.
    #[serde(default)]
    pub agent_models: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AgentUpdate {
    Text {
        text: String,
    },
    Thought {
        text: String,
    },
    Tool {
        tool_call_id: String,
        title: String,
        status: ToolCallStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_kind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    Done {
        stop_reason: String,
    },
    Error {
        message: String,
    },
    /// The models the agent offers for this session and the one in use (sent when the session
    /// starts and whenever the model changes).
    Models {
        current: Option<String>,
        options: Vec<ModelChoice>,
    },
}

/// One model an agent offers: `value` is what the agent is told, `name` what the user reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChoice {
    pub value: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionOption {
    pub id: String,
    pub label: String,
    pub kind: PermissionOptionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub chat_id: String,
    pub request_id: String,
    pub title: String,
    pub commands: Vec<String>,
    pub options: Vec<PermissionOption>,
}

/// A question the agent asks the user through ACP (`elicitation/create`, form mode): a message
/// and the JSON Schema of the answer (`properties` with `oneOf` options, multi-select arrays,
/// free text). The UI draws it as a card; the answer goes back with `agent_respond_question`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentQuestion {
    pub chat_id: String,
    pub request_id: String,
    pub message: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub duration_sec: f64,
    pub fps: f64,
    pub frames: u64,
    pub width: u32,
    pub height: u32,
}

/// A file on disk as the UI needs it: a render, or an image the agent made.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    /// RFC 3339 UTC with milliseconds.
    pub modified_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn video_format_uses_ratio_literals() {
        assert_eq!(serde_json::to_value(VideoFormat::Portrait).unwrap(), json!("9:16"));
        let f: VideoFormat = serde_json::from_value(json!("16:9")).unwrap();
        assert_eq!(f, VideoFormat::Landscape);
    }

    #[test]
    fn tool_id_and_job_kind_literals() {
        assert_eq!(serde_json::to_value(ToolId::ElevenlabsMcp).unwrap(), json!("elevenlabs-mcp"));
        assert_eq!(serde_json::to_value(JobKind::StudyStyle).unwrap(), json!("study-style"));
        assert_eq!(serde_json::to_value(JobStatus::NeedsYou).unwrap(), json!("needs-you"));
    }

    #[test]
    fn message_part_frame_round_trips() {
        let v = json!({"type":"frame","projectId":"p1","frame":12,"timeSec":0.4,
            "imagePath":"/x.png","marks":[{"x":0.5,"y":0.25}]});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::Frame { frame: 12, .. }));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
    }

    #[test]
    fn message_part_a2ui_round_trips() {
        let v = json!({"type":"a2ui","surfaceId":"s","messages":[]});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::A2ui { surface_id, .. } if surface_id == "s"));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
        let v = json!({"type":"a2ui-action","summary":"Picked: A","action":{"version":"v0.9"}});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::A2uiAction { summary, .. } if summary == "Picked: A"));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
    }

    #[test]
    fn message_part_style_mention_round_trips() {
        let v = json!({"type":"style-mention","styleId":"builtin-atlas","name":"Atlas journey"});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::StyleMention { style_id, .. } if style_id == "builtin-atlas"));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
    }

    #[test]
    fn message_part_file_round_trips() {
        let v = json!({"type":"file","path":"/w/attachments/a.pdf","name":"a.pdf",
            "mime":"application/pdf","size":2048});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::File { size: 2048, modified_at: None, .. }));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
        // An image the agent made keeps its time.
        let v = json!({"type":"file","path":"/w/p/a.png","name":"a.png","mime":"image/png","size":3,
            "modifiedAt":"2026-11-04T10:00:00.000Z"});
        let part: MessagePart = serde_json::from_value(v.clone()).unwrap();
        assert!(matches!(&part, MessagePart::File { modified_at: Some(_), .. }));
        assert_eq!(serde_json::to_value(&part).unwrap(), v);
    }

    #[test]
    fn agent_update_tool_round_trips() {
        let v = json!({"kind":"tool","toolCallId":"t1","title":"Run","status":"running"});
        let u: AgentUpdate = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&u).unwrap(), v);
        let d = json!({"kind":"done","stopReason":"end_turn"});
        let u: AgentUpdate = serde_json::from_value(d.clone()).unwrap();
        assert_eq!(serde_json::to_value(&u).unwrap(), d);
    }

    #[test]
    fn tool_kind_and_path_are_optional_and_camel_case() {
        let v = json!({"kind":"tool","toolCallId":"t1","title":"Read","status":"done","toolKind":"read","path":"/a/b.ts"});
        let u: AgentUpdate = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&u).unwrap(), v);
        let part = json!({"type":"tool","toolCallId":"t1","title":"Read","status":"done","toolKind":"read","path":"/a/b.ts"});
        let p: MessagePart = serde_json::from_value(part.clone()).unwrap();
        assert_eq!(serde_json::to_value(&p).unwrap(), part);
        let t = json!({"type":"thought","text":"hmm"});
        let p: MessagePart = serde_json::from_value(t.clone()).unwrap();
        assert_eq!(serde_json::to_value(&p).unwrap(), t);
    }

    #[test]
    fn style_notes_are_partial_and_new_project_style_optional() {
        let v = json!({"id":"s","name":"n","source":"builtin","sourceUrl":null,
            "notes":{"camera":"slow push"},"prompt":"p","createdAt":"t"});
        let s: Style = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(serde_json::to_value(&s).unwrap(), v);
        let np: NewProject = serde_json::from_value(
            json!({"title":"t","kind":"promo","format":"1:1","language":"id"}),
        )
        .unwrap();
        assert_eq!(np.style_id, None);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub used_percent: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub label: String,
    pub window: UsageWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsageStatus {
    Ok,
    SignedOut,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsage {
    pub agent_id: AgentId,
    pub status: UsageStatus,
    pub session: Option<UsageWindow>,
    pub weekly: Option<UsageWindow>,
    pub model_weekly: Option<ModelUsage>,
    pub updated_at: String,
    pub message: Option<String>,
    /// For agents without plan limits (opencode): what they used today and over the last 7 days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend: Option<SpendSummary>,
}

/// Replies, tokens (input, output and reasoning; cached reads left out) and cost over a period.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spend {
    pub replies: u64,
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendSummary {
    pub today: Spend,
    pub week: Spend,
}
