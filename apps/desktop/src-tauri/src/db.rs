//! SQLite storage for settings, projects, chats, styles and jobs.

use crate::types::*;
use rusqlite::types::{Value as Sql, ValueRef};
use rusqlite::{params_from_iter, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub struct DbError(pub String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError(e.to_string())
    }
}
impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        DbError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DbError>;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY, title TEXT NOT NULL, slug TEXT NOT NULL, folder TEXT NOT NULL,
  kind TEXT NOT NULL, format TEXT NOT NULL, language TEXT NOT NULL, status TEXT NOT NULL,
  style_id TEXT, duration_sec REAL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS chats (
  id TEXT PRIMARY KEY, project_id TEXT, agent_id TEXT NOT NULL,
  title TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY, chat_id TEXT NOT NULL, role TEXT NOT NULL,
  parts TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS messages_chat_at ON messages(chat_id, created_at);
DROP INDEX IF EXISTS messages_chat;
CREATE TABLE IF NOT EXISTS styles (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, source TEXT NOT NULL, source_url TEXT,
  notes TEXT NOT NULL, prompt TEXT NOT NULL, created_at TEXT NOT NULL,
  engine TEXT, preview_url TEXT, preview_video_url TEXT, template TEXT, version INTEGER, based_on TEXT
);
CREATE TABLE IF NOT EXISTS jobs (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, project_id TEXT, chat_id TEXT,
  agent_id TEXT NOT NULL, status TEXT NOT NULL, progress REAL, note TEXT, source_url TEXT, updated_at TEXT NOT NULL,
  render_baseline_at TEXT
);
";

// Column lists use camelCase aliases so rows deserialize straight into the serde types.
const PROJECT_SELECT: &str = "SELECT id, title, slug, folder, kind, format, language, status, \
    style_id AS styleId, duration_sec AS durationSec, \
    created_at AS createdAt, updated_at AS updatedAt \
    FROM projects";
const PROJECT_KEYS: [&str; 12] = [
    "id", "title", "slug", "folder", "kind", "format", "language", "status", "styleId",
    "durationSec", "createdAt", "updatedAt",
];
const CHAT_SELECT: &str = "SELECT id, project_id AS projectId, agent_id AS agentId, title, \
    created_at AS createdAt FROM chats";
const JOB_SELECT: &str = "SELECT id, kind, title, project_id AS projectId, chat_id AS chatId, \
    agent_id AS agentId, status, progress, note, source_url AS sourceUrl, updated_at AS updatedAt, \
    render_baseline_at AS renderBaselineAt, archived_at AS archivedAt FROM jobs";
const JOB_KEYS: [&str; 12] = [
    "id", "kind", "title", "projectId", "chatId", "agentId", "status", "progress", "note",
    "sourceUrl", "updatedAt", "renderBaselineAt",
];

/// Ids of legacy built-ins from older versions that are no longer bundled.
const LEGACY_BUILTIN_IDS: [&str; 3] =
    ["builtin-bold-explainer", "builtin-kinetic-quote", "builtin-news-recap"];

/// Built-in Opuscar styles without previewUrl/previewVideoUrl that were pruned.
const DROPPED_BUILTIN_IDS: [&str; 41] = [
    "art-deco",
    "ascii-crt",
    "backrooms",
    "blueprint",
    "brick-toy",
    "cel-anime-80s",
    "circuit-board",
    "dark-keynote",
    "data-physical",
    "desktop-flythrough",
    "engraving",
    "game-show",
    "glass-product",
    "halftone-dossier",
    "hd-2d",
    "hologram-hud",
    "impasto",
    "ink-wash",
    "isometric-picture-book",
    "living-screencast",
    "lowpoly-island",
    "memphis-diorama",
    "microgame",
    "microscope",
    "paper-lantern",
    "paper-popup",
    "pictogram-motion",
    "pixel-rpg",
    "printed-long-take",
    "risograph",
    "rubber-hose",
    "scifi-toon",
    "silent-film",
    "silkscreen-poster",
    "spy-titles",
    "stained-glass",
    "teardown",
    "tilt-shift",
    "ukiyoe",
    "white-model",
    "woodcut",
];

/// 28 bundled built-in styles (3 original styles + 25 curated Opuscar styles with active previews).
pub fn bundled_builtin_styles() -> &'static [Style] {
    static STYLES: OnceLock<Vec<Style>> = OnceLock::new();
    STYLES.get_or_init(|| {
        serde_json::from_str(include_str!("builtin_styles.json"))
            .expect("bundled builtin_styles.json is valid")
    })
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn sql_from_json(v: &Value) -> Sql {
    match v {
        Value::Null => Sql::Null,
        Value::Bool(b) => Sql::Integer(*b as i64),
        Value::Number(n) => match n.as_i64() {
            Some(i) => Sql::Integer(i),
            None => Sql::Real(n.as_f64().unwrap_or_default()),
        },
        Value::String(s) => Sql::Text(s.clone()),
        other => Sql::Text(other.to_string()),
    }
}

fn json_from_ref(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(_) => Value::Null,
    }
}

/// Serializes `item` and returns its values in `keys` order, ready to bind.
fn bind_values<T: serde::Serialize>(item: &T, keys: &[&str]) -> Result<Vec<Sql>> {
    let v = serde_json::to_value(item)?;
    Ok(keys.iter().map(|k| sql_from_json(v.get(*k).unwrap_or(&Value::Null))).collect())
}

fn query_as<T: DeserializeOwned>(
    conn: &Connection,
    sql: &str,
    args: &[&dyn rusqlite::ToSql],
) -> Result<Vec<T>> {
    let mut stmt = conn.prepare(sql)?;
    let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let rows = stmt.query_map(args, |row| {
        let mut obj = Map::new();
        for (i, n) in names.iter().enumerate() {
            obj.insert(n.clone(), json_from_ref(row.get_ref(i)?));
        }
        Ok(Value::Object(obj))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(serde_json::from_value(r?)?);
    }
    Ok(out)
}

fn merge_patch(target: &mut Value, patch: &Value) {
    if let (Value::Object(t), Value::Object(p)) = (target, patch) {
        for (k, v) in p {
            t.insert(k.clone(), v.clone());
        }
    }
}

pub struct Db {
    conn: Mutex<Connection>,
    home: Mutex<std::path::PathBuf>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Db> {
        let db = Self::init(Connection::open(path)?)?;
        db.seed_builtin_styles()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Db> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Db> {
        conn.execute_batch(SCHEMA)?;
        Self::cleanup_calendar_and_autopilot(&conn)?;
        Self::migrate_styles(&conn)?;
        Self::migrate_jobs(&conn)?;
        Self::migrate_archive(&conn)?;
        Ok(Db { conn: Mutex::new(conn), home: Mutex::new(Default::default()) })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ---- settings ----

    fn default_settings(home: &Path) -> Value {
        json!({
            "workingFolder": home.join("Listune").to_string_lossy(),
            "defaultAgent": "claude",
            "frameCheck": true,
            "language": "id",
            "format": "9:16",
            "fps": 30,
            "onboarded": false,
            "autoMode": false,
            "uiLanguage": null,
            "backgroundMode": false,
            "openAtLogin": false,
            "buildJobsChecked": false,
            "projectStatusChecked": false,
            "agentModels": {},
        })
    }

    fn settings_value(&self, home: &Path) -> Result<Value> {
        let mut v = Self::default_settings(home);
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (k, raw) = row?;
            if let (Ok(val), Some(obj)) = (serde_json::from_str::<Value>(&raw), v.as_object_mut()) {
                obj.insert(k, val);
            }
        }
        Ok(v)
    }

    /// Stored values over defaults; a stored value that no longer parses falls back to defaults.
    pub fn get_settings(&self, home: &Path) -> Settings {
        *self.home.lock().unwrap_or_else(|e| e.into_inner()) = home.to_path_buf();
        self.settings_value(home)
            .and_then(|v| Ok(serde_json::from_value(v)?))
            .unwrap_or_else(|_| {
                serde_json::from_value(Self::default_settings(home)).expect("valid defaults")
            })
    }

    /// Merges `patch` over the current settings, validates, and persists only the patched
    /// keys, so unset keys (like the home-based working folder) keep following their defaults.
    /// The returned value uses the home from the last `get_settings` call for unset defaults.
    pub fn update_settings(&self, patch: Value) -> Result<Settings> {
        if !patch.is_object() {
            return Err(DbError("settings patch must be a JSON object".into()));
        }
        let home = self.home.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let mut merged = self.settings_value(&home)?;
        merge_patch(&mut merged, &patch);
        let settings: Settings = serde_json::from_value(merged)?;
        let full = serde_json::to_value(&settings)?;
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        for k in patch.as_object().into_iter().flat_map(|o| o.keys()) {
            let Some(v) = full.get(k) else { continue };
            tx.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![k, v.to_string()],
            )?;
        }
        tx.commit()?;
        Ok(settings)
    }

    // ---- projects ----

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let sql = format!("{PROJECT_SELECT} ORDER BY updated_at DESC, rowid DESC");
        query_as(&self.conn(), &sql, &[])
    }

    pub fn insert_project(&self, p: &Project) -> Result<()> {
        let vals = bind_values(p, &PROJECT_KEYS)?;
        self.conn().execute(
            "INSERT INTO projects (id, title, slug, folder, kind, format, language, status, \
             style_id, duration_sec, created_at, updated_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params_from_iter(vals),
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn().execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let sql = format!("{PROJECT_SELECT} WHERE id = ?1");
        Ok(query_as::<Project>(&self.conn(), &sql, &[&id])?.into_iter().next())
    }

    /// Shallow-merges `patch` over the stored project. `id` cannot change; `updatedAt`
    /// is bumped unless the patch sets it.
    pub fn update_project(&self, id: &str, patch: Value) -> Result<Project> {
        if let Some(new_id) = patch.get("id") {
            if new_id != &json!(id) {
                return Err(DbError("project id cannot be changed".into()));
            }
        }
        let current = self
            .get_project(id)?
            .ok_or_else(|| DbError(format!("project not found: {id}")))?;
        let mut merged = serde_json::to_value(&current)?;
        if patch.get("updatedAt").is_none() {
            merged["updatedAt"] = json!(now());
        }
        merge_patch(&mut merged, &patch);
        let updated: Project = serde_json::from_value(merged)?;
        let mut vals = bind_values(&updated, &PROJECT_KEYS)?;
        let id_val = vals.remove(0);
        vals.push(id_val);
        self.conn().execute(
            "UPDATE projects SET title=?1, slug=?2, folder=?3, kind=?4, format=?5, language=?6, \
             status=?7, style_id=?8, duration_sec=?9, created_at=?10, updated_at=?11 \
             WHERE id=?12",
            params_from_iter(vals),
        )?;
        Ok(updated)
    }

    /// Moves a project along draft → building → rendered. Without `reset` it never goes back, and a
    /// project already there is returned as it is, unwritten. `updatedAt` never changes.
    pub fn set_project_status(&self, id: &str, status: ProjectStatus, reset: bool) -> Result<Project> {
        let rank = |s: ProjectStatus| match s {
            ProjectStatus::Draft => 0,
            ProjectStatus::Building => 1,
            ProjectStatus::Rendered => 2,
        };
        let current = self
            .get_project(id)?
            .ok_or_else(|| DbError(format!("project not found: {id}")))?;
        let moves = if reset { current.status != status } else { rank(status) > rank(current.status) };
        if !moves {
            return Ok(current);
        }
        // Not an edit: `updatedAt` (the "recent" order and "x ago") stays as it was.
        self.update_project(id, json!({ "status": status, "updatedAt": current.updated_at }))
    }

    // ---- chats and messages ----

    pub fn create_chat(&self, project_id: Option<&str>, agent: AgentId, title: &str) -> Result<Chat> {
        let chat = Chat {
            id: new_id(),
            project_id: project_id.map(str::to_string),
            agent_id: agent,
            title: title.to_string(),
            created_at: now(),
        };
        let vals = bind_values(&chat, &["id", "projectId", "agentId", "title", "createdAt"])?;
        self.conn().execute(
            "INSERT INTO chats (id, project_id, agent_id, title, created_at) VALUES (?1,?2,?3,?4,?5)",
            params_from_iter(vals),
        )?;
        Ok(chat)
    }

    /// `Some(id)` lists that project's chats; `None` lists chats not tied to a project.
    pub fn list_chats(&self, project_id: Option<&str>) -> Result<Vec<Chat>> {
        let conn = self.conn();
        match project_id {
            Some(p) => query_as(
                &conn,
                &format!("{CHAT_SELECT} WHERE project_id = ?1 ORDER BY created_at DESC, rowid DESC"),
                &[&p],
            ),
            None => query_as(
                &conn,
                &format!("{CHAT_SELECT} WHERE project_id IS NULL ORDER BY created_at DESC, rowid DESC"),
                &[],
            ),
        }
    }

    /// Chats, project or not, newest activity first; a project once (its most active chat), at most
    /// `limit` entries. Chats of a deleted project are left out.
    pub fn list_recent_chats(&self, limit: Option<u32>) -> Result<Vec<RecentChat>> {
        let limit = limit.map_or(-1, i64::from);
        query_as(
            &self.conn(),
            "WITH active AS ( \
               SELECT c.id, c.project_id, c.agent_id, c.title, c.created_at, c.rowid AS rid, p.title AS project_title, \
                 COALESCE((SELECT MAX(m.created_at) FROM messages m WHERE m.chat_id = c.id), c.created_at) AS active_at \
               FROM chats c LEFT JOIN projects p ON p.id = c.project_id \
               WHERE (c.project_id IS NULL OR p.id IS NOT NULL) AND c.archived_at IS NULL \
             ), ranked AS ( \
               SELECT *, ROW_NUMBER() OVER (PARTITION BY COALESCE(project_id, id) ORDER BY active_at DESC, rid DESC) AS n \
               FROM active \
             ) \
             SELECT id, project_id AS projectId, agent_id AS agentId, title, created_at AS createdAt, \
               project_title AS projectTitle, active_at AS activeAt \
             FROM ranked WHERE n = 1 ORDER BY active_at DESC, rid DESC LIMIT ?1",
            &[&limit],
        )
    }

    /// Archived threads, most recently archived first: a project once, like Recent.
    pub fn list_archived_chats(&self) -> Result<Vec<RecentChat>> {
        query_as(
            &self.conn(),
            "WITH arch AS ( \
               SELECT c.id, c.project_id, c.agent_id, c.title, c.created_at, c.archived_at, c.rowid AS rid, \
                 p.title AS project_title, \
                 COALESCE((SELECT MAX(m.created_at) FROM messages m WHERE m.chat_id = c.id), c.created_at) AS active_at \
               FROM chats c LEFT JOIN projects p ON p.id = c.project_id \
               WHERE (c.project_id IS NULL OR p.id IS NOT NULL) AND c.archived_at IS NOT NULL \
             ), ranked AS ( \
               SELECT *, ROW_NUMBER() OVER (PARTITION BY COALESCE(project_id, id) ORDER BY active_at DESC, rid DESC) AS n \
               FROM arch \
             ) \
             SELECT id, project_id AS projectId, agent_id AS agentId, title, created_at AS createdAt, \
               project_title AS projectTitle, active_at AS activeAt, archived_at AS archivedAt \
             FROM ranked WHERE n = 1 ORDER BY archived_at DESC, rid DESC",
            &[],
        )
    }

    /// Archives or restores a whole thread: every chat of the chat's project, or the chat alone.
    pub fn set_thread_archived(&self, chat_id: &str, archived: bool) -> Result<()> {
        let stamp: Option<String> = archived.then(now);
        self.conn().execute(
            "UPDATE chats SET archived_at = ?2 \
             WHERE COALESCE(project_id, id) = (SELECT COALESCE(project_id, id) FROM chats WHERE id = ?1)",
            rusqlite::params![chat_id, stamp],
        )?;
        Ok(())
    }

    /// Archives or restores board cards. A working card is never archived: it is still going.
    pub fn set_jobs_archived(&self, ids: &[String], archived: bool) -> Result<()> {
        let stamp: Option<String> = archived.then(now);
        let conn = self.conn();
        let tx = conn.unchecked_transaction()?;
        for id in ids {
            tx.execute(
                "UPDATE jobs SET archived_at = ?2 WHERE id = ?1 AND (?2 IS NULL OR status <> 'working')",
                rusqlite::params![id, stamp],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Deletes everything archived for good: archived chats with their messages, and archived cards.
    /// Projects and their folders (the videos) stay. Returns (chats, cards) removed.
    pub fn purge_archive(&self) -> Result<(usize, usize)> {
        let conn = self.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute("DELETE FROM messages WHERE chat_id IN (SELECT id FROM chats WHERE archived_at IS NOT NULL)", [])?;
        let chats = tx.execute("DELETE FROM chats WHERE archived_at IS NOT NULL", [])?;
        let cards = tx.execute("DELETE FROM jobs WHERE archived_at IS NOT NULL AND status <> 'working'", [])?;
        tx.commit()?;
        Ok((chats, cards))
    }

    /// Points a chat at another agent. `None` when the chat doesn't exist.
    pub fn set_chat_agent(&self, chat_id: &str, agent: AgentId) -> Result<Option<Chat>> {
        let v = serde_json::to_value(agent)?;
        self.conn().execute(
            "UPDATE chats SET agent_id = ?1 WHERE id = ?2",
            rusqlite::params![v.as_str().unwrap_or_default(), chat_id],
        )?;
        self.get_chat(chat_id)
    }

    pub fn get_chat(&self, id: &str) -> Result<Option<Chat>> {
        let sql = format!("{CHAT_SELECT} WHERE id = ?1");
        Ok(query_as::<Chat>(&self.conn(), &sql, &[&id])?.into_iter().next())
    }

    pub fn add_message(
        &self,
        chat_id: &str,
        role: MessageRole,
        parts: &[MessagePart],
    ) -> Result<ChatMessage> {
        let msg = ChatMessage {
            id: new_id(),
            chat_id: chat_id.to_string(),
            role,
            parts: parts.to_vec(),
            created_at: now(),
        };
        let role_s = serde_json::to_value(role)?;
        self.conn().execute(
            "INSERT INTO messages (id, chat_id, role, parts, created_at) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![
                msg.id,
                msg.chat_id,
                role_s.as_str().unwrap_or_default(),
                serde_json::to_string(&msg.parts)?,
                msg.created_at
            ],
        )?;
        // Writing in an archived thread brings it back to Recent.
        if role == MessageRole::User {
            self.set_thread_archived(chat_id, false)?;
        }
        Ok(msg)
    }

    pub fn list_messages(&self, chat_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, chat_id AS chatId, role, parts, created_at AS createdAt \
             FROM messages WHERE chat_id = ?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map([chat_id], |r| {
            Ok(json!({
                "id": r.get::<_, String>(0)?,
                "chatId": r.get::<_, String>(1)?,
                "role": r.get::<_, String>(2)?,
                "parts": r.get::<_, String>(3)?,
                "createdAt": r.get::<_, String>(4)?,
            }))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let mut v = row?;
            let parts: Value = serde_json::from_str(v["parts"].as_str().unwrap_or("[]"))?;
            v["parts"] = parts;
            out.push(serde_json::from_value(v)?);
        }
        Ok(out)
    }

    // ---- styles ----

    /// Adds the API-backed columns to databases created before them and drops the old
    /// hard-coded built-ins (the API owns built-ins now).
    fn migrate_styles(conn: &Connection) -> Result<()> {
        let mut have: Vec<String> = Vec::new();
        {
            let mut stmt = conn.prepare("PRAGMA table_info(styles)")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
            for r in rows {
                have.push(r?);
            }
        }
        for (col, ty) in [
            ("engine", "TEXT"),
            ("preview_url", "TEXT"),
            ("preview_video_url", "TEXT"),
            ("template", "TEXT"),
            ("version", "INTEGER"),
            ("based_on", "TEXT"),
            ("build", "TEXT"),
            ("palette", "TEXT"),
            ("refs", "TEXT"),
        ] {
            if !have.iter().any(|c| c == col) {
                conn.execute_batch(&format!("ALTER TABLE styles ADD COLUMN {col} {ty}"))?;
            }
        }
        for id in LEGACY_BUILTIN_IDS {
            conn.execute("DELETE FROM styles WHERE id = ?1 AND source = 'builtin'", [id])?;
        }
        for id in DROPPED_BUILTIN_IDS {
            conn.execute(
                "DELETE FROM styles WHERE id = ?1 AND source = 'builtin' \
                 AND id NOT IN (SELECT based_on FROM styles WHERE based_on IS NOT NULL)",
                [id],
            )?;
        }
        Ok(())
    }

    /// Drops legacy Calendar/Autopilot/auth/telemetry/API state from existing databases safely.
    fn cleanup_calendar_and_autopilot(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "UPDATE projects SET status = 'rendered' WHERE status = 'scheduled';
             DROP TABLE IF EXISTS autopilot_slots;
             DROP TABLE IF EXISTS autopilots;
             DELETE FROM settings WHERE key LIKE 'authAccount:%'
                 OR key IN ('apiUrl', 'installId', 'shareInstallStats', 'lastPingAt');",
        )?;
        let mut stmt = conn.prepare("PRAGMA table_info(projects)")?;
        let have = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(stmt);
        for col in ["scheduled_at", "review_state", "posted_at", "reminded_at", "review_reminded_at"] {
            if have.iter().any(|c| c == col) {
                conn.execute_batch(&format!("ALTER TABLE projects DROP COLUMN {col}"))?;
            }
        }
        Ok(())
    }

    /// Adds `archived_at` to chats and jobs in databases created before the archive.
    fn migrate_archive(conn: &Connection) -> Result<()> {
        for table in ["chats", "jobs"] {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
            let have = stmt
                .query_map([], |r| r.get::<_, String>(1))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            drop(stmt);
            if !have.iter().any(|c| c == "archived_at") {
                conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN archived_at TEXT"))?;
            }
        }
        Ok(())
    }

    /// Adds `jobs.source_url` and `jobs.render_baseline_at` to databases created before them.
    fn migrate_jobs(conn: &Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(jobs)")?;
        let have = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(stmt);
        if !have.iter().any(|c| c == "source_url") {
            conn.execute_batch("ALTER TABLE jobs ADD COLUMN source_url TEXT")?;
        }
        if !have.iter().any(|c| c == "render_baseline_at") {
            conn.execute_batch("ALTER TABLE jobs ADD COLUMN render_baseline_at TEXT")?;
        }
        Ok(())
    }

    fn insert_or_replace_builtin_tx(tx: &rusqlite::Transaction<'_>, s: &Style) -> Result<()> {
        let existing: Option<String> =
            tx.query_row("SELECT source FROM styles WHERE id = ?1", [&s.id], |r| r.get(0)).optional()?;
        if existing.as_deref().is_some_and(|src| src != "builtin") {
            // Not a built-in: this one belongs to the user.
            eprintln!("builtin styles: skipping \"{}\", a style of that id already exists", s.id);
            return Ok(());
        }
        let bundled = bundled_builtin_styles().iter().find(|b| b.id == s.id);
        let name = if s.name.trim().is_empty() {
            bundled.map(|b| b.name.as_str()).unwrap_or("")
        } else {
            s.name.as_str()
        };
        let source_url = s
            .source_url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
            .or_else(|| bundled.and_then(|b| b.source_url.as_deref()));
        let prompt = if s.prompt.trim().is_empty() {
            bundled.map(|b| b.prompt.as_str()).unwrap_or("")
        } else {
            s.prompt.as_str()
        };
        let created_at = if s.created_at.trim().is_empty() {
            bundled.map(|b| b.created_at.as_str()).unwrap_or("")
        } else {
            s.created_at.as_str()
        };
        let template_opt = s.template.as_ref().or_else(|| bundled.and_then(|b| b.template.as_ref()));
        let template = match template_opt {
            Some(t) => Some(serde_json::to_string(t)?),
            None => None,
        };
        // Keep the bundled engine when a bundled starter template is preserved over an existing payload that omits template.
        let engine_val = if s.template.is_none() && bundled.and_then(|b| b.template.as_ref()).is_some() {
            bundled.and_then(|b| b.engine).or(s.engine)
        } else {
            s.engine.or_else(|| bundled.and_then(|b| b.engine))
        };
        let engine = match engine_val {
            Some(e) => serde_json::to_value(e)?.as_str().map(str::to_string),
            None => None,
        };
        let preview_url = s
            .preview_url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
            .or_else(|| bundled.and_then(|b| b.preview_url.as_deref()));
        let preview_video_url = s
            .preview_video_url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
            .or_else(|| bundled.and_then(|b| b.preview_video_url.as_deref()));
        let version = s.version.or_else(|| bundled.and_then(|b| b.version));
        let based_on = s
            .based_on
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .or_else(|| bundled.and_then(|b| b.based_on.as_deref()));
        let build = s
            .build
            .as_deref()
            .filter(|b| !b.trim().is_empty())
            .or_else(|| bundled.and_then(|b| b.build.as_deref()));
        let palette_list = if s.palette.is_empty() {
            bundled.map(|b| b.palette.as_slice()).unwrap_or_default()
        } else {
            &s.palette
        };
        let palette = if palette_list.is_empty() {
            None
        } else {
            Some(serde_json::to_string(palette_list)?)
        };
        let refs = if s.references.is_empty() {
            bundled.and_then(|b| {
                if b.references.is_empty() {
                    None
                } else {
                    serde_json::to_string(&b.references).ok()
                }
            })
        } else {
            Some(serde_json::to_string(&s.references)?)
        };
        let notes_merged = if let Some(b) = bundled {
            let mut merged = b.notes.clone();
            for (k, v) in &s.notes {
                if !v.trim().is_empty() {
                    merged.insert(*k, v.clone());
                }
            }
            merged
        } else {
            s.notes.clone()
        };
        tx.execute(
            "INSERT INTO styles (id, name, source, source_url, notes, prompt, created_at, \
             engine, preview_url, preview_video_url, template, version, based_on, build, palette, refs) \
             VALUES (?1,?2,'builtin',?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15) \
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, source='builtin', \
             source_url=excluded.source_url, notes=excluded.notes, prompt=excluded.prompt, \
             created_at=excluded.created_at, engine=excluded.engine, \
             preview_url=COALESCE(excluded.preview_url, NULLIF(TRIM(styles.preview_url), '')), \
             preview_video_url=COALESCE(excluded.preview_video_url, NULLIF(TRIM(styles.preview_video_url), '')), \
             template=excluded.template, version=excluded.version, based_on=excluded.based_on, \
             build=excluded.build, palette=excluded.palette, refs=excluded.refs",
            rusqlite::params![
                s.id,
                name,
                source_url,
                serde_json::to_string(&notes_merged)?,
                prompt,
                created_at,
                engine,
                preview_url,
                preview_video_url,
                template,
                version,
                based_on,
                build,
                palette,
                refs,
            ],
        )?;
        Ok(())
    }

    fn merge_bundled_backfill(existing: &Style, bundled: &Style) -> Option<Style> {
        let mut merged = existing.clone();
        if merged.name.trim().is_empty() && !bundled.name.trim().is_empty() {
            merged.name = bundled.name.clone();
        }
        if merged.source_url.as_deref().is_none_or(|u| u.trim().is_empty()) {
            merged.source_url = bundled.source_url.clone();
        }
        for (k, v) in &bundled.notes {
            if !v.trim().is_empty() && merged.notes.get(k).is_none_or(|ev| ev.trim().is_empty()) {
                merged.notes.insert(*k, v.clone());
            }
        }
        if merged.prompt.trim().is_empty() && !bundled.prompt.trim().is_empty() {
            merged.prompt = bundled.prompt.clone();
        }
        if merged.created_at.trim().is_empty() && !bundled.created_at.trim().is_empty() {
            merged.created_at = bundled.created_at.clone();
        }
        let restored_template = merged.template.is_none() && bundled.template.is_some();
        if restored_template {
            merged.template = bundled.template.clone();
        }
        if (restored_template && bundled.engine.is_some()) || merged.engine.is_none() {
            merged.engine = bundled.engine;
        }
        if merged.preview_url.as_deref().is_none_or(|u| u.trim().is_empty()) {
            merged.preview_url = bundled.preview_url.clone();
        }
        if merged.preview_video_url.as_deref().is_none_or(|u| u.trim().is_empty()) {
            merged.preview_video_url = bundled.preview_video_url.clone();
        }
        if merged.version.is_none() {
            merged.version = bundled.version;
        }
        if merged.based_on.as_deref().is_none_or(|id| id.trim().is_empty()) {
            merged.based_on = bundled.based_on.clone();
        }
        if merged.build.as_deref().is_none_or(|b| b.trim().is_empty()) {
            merged.build = bundled.build.clone();
        }
        if merged.palette.is_empty() {
            merged.palette = bundled.palette.clone();
        }
        if merged.references.is_empty() {
            merged.references = bundled.references.clone();
        }
        (merged != *existing).then_some(merged)
    }

    /// Seeds the 28 bundled built-in styles (3 original + 25 active Opuscar) into the database.
    /// Missing bundled styles are inserted; existing built-in styles only have blank/missing
    /// fields backfilled so existing prompts, engines, previews, and templates are never clobbered.
    /// Unused dropped built-in styles are also removed cleanly.
    pub fn seed_builtin_styles(&self) -> Result<()> {
        let mut conn = self.conn();
        let existing_list = Self::list_styles_conn(&conn)?;
        let existing_map: std::collections::HashMap<String, Style> = existing_list
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

        let mut to_insert: Vec<&Style> = Vec::new();
        let mut to_backfill: Vec<Style> = Vec::new();
        for b in bundled_builtin_styles() {
            match existing_map.get(&b.id) {
                None => to_insert.push(b),
                Some(existing) if existing.source != StyleSource::Builtin => {
                    // User style owns this id.
                }
                Some(existing) => {
                    if let Some(updated) = Self::merge_bundled_backfill(existing, b) {
                        to_backfill.push(updated);
                    }
                }
            }
        }
        let has_dropped = DROPPED_BUILTIN_IDS
            .iter()
            .any(|id| existing_map.get(*id).is_some_and(|s| s.source == StyleSource::Builtin));

        if to_insert.is_empty() && to_backfill.is_empty() && !has_dropped {
            return Ok(());
        }

        let tx = conn.transaction()?;
        if has_dropped {
            for id in DROPPED_BUILTIN_IDS {
                tx.execute(
                    "DELETE FROM styles WHERE id = ?1 AND source = 'builtin' \
                     AND id NOT IN (SELECT based_on FROM styles WHERE based_on IS NOT NULL)",
                    [id],
                )?;
            }
        }
        for s in to_insert {
            Self::insert_or_replace_builtin_tx(&tx, s)?;
        }
        for s in to_backfill {
            Self::insert_or_replace_builtin_tx(&tx, &s)?;
        }
        tx.commit()?;
        Ok(())
    }

    #[cfg(test)]
    pub fn seed_rows_for_test(&self, styles: &[Style]) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        for s in styles {
            Self::insert_or_replace_builtin_tx(&tx, s)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_style(&self, id: &str) -> Result<Option<Style>> {
        Ok(self.list_styles()?.into_iter().find(|s| s.id == id))
    }

    fn list_styles_conn(conn: &Connection) -> Result<Vec<Style>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, source, source_url AS sourceUrl, notes, prompt, created_at AS createdAt, \
             engine, preview_url AS previewUrl, preview_video_url AS previewVideoUrl, template, version, based_on AS basedOn, build, palette, refs AS \"references\" FROM styles \
             ORDER BY CASE source WHEN 'builtin' THEN 0 ELSE 1 END, \
             CASE source WHEN 'builtin' THEN rowid END, created_at, rowid",
        )?;
        let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let rows = stmt.query_map([], |row| {
            let mut obj = Map::new();
            for (i, n) in names.iter().enumerate() {
                obj.insert(n.clone(), json_from_ref(row.get_ref(i)?));
            }
            Ok(Value::Object(obj))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let mut v = row?;
            let notes: Value = serde_json::from_str(v["notes"].as_str().unwrap_or("{}"))?;
            v["notes"] = notes;
            if let Some(t) = v["template"].as_str().map(str::to_string) {
                v["template"] = if t.trim().is_empty() { Value::Null } else { serde_json::from_str(&t)? };
            }
            // JSON arrays in TEXT columns; empty or unreadable means none.
            for key in ["palette", "references"] {
                let list = v[key].as_str().and_then(|t| serde_json::from_str::<Value>(t).ok()).unwrap_or(Value::Array(vec![]));
                v[key] = list;
            }
            out.push(serde_json::from_value(v)?);
        }
        Ok(out)
    }

    pub fn list_styles(&self) -> Result<Vec<Style>> {
        let conn = self.conn();
        Self::list_styles_conn(&conn)
    }

    /// Inserts or replaces a style. Built-in styles are read-only.
    pub fn save_style(&self, s: &Style) -> Result<()> {
        let conn = self.conn();
        if Self::is_builtin(&conn, &s.id)? || s.source == StyleSource::Builtin {
            return Err(DbError("built-in styles cannot be changed".into()));
        }
        let source = serde_json::to_value(s.source)?;
        let engine = match s.engine {
            Some(e) => serde_json::to_value(e)?.as_str().map(str::to_string),
            None => None,
        };
        let template = match &s.template {
            Some(t) => Some(serde_json::to_string(t)?),
            None => None,
        };
        conn.execute(
            "INSERT INTO styles (id, name, source, source_url, notes, prompt, created_at, engine, preview_url, \
             preview_video_url, template, version, based_on, build, palette, refs) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16) ON CONFLICT(id) DO UPDATE SET name=excluded.name, \
             source=excluded.source, source_url=excluded.source_url, notes=excluded.notes, \
             prompt=excluded.prompt, created_at=excluded.created_at, engine=excluded.engine, \
             preview_url=excluded.preview_url, preview_video_url=excluded.preview_video_url, \
             template=excluded.template, version=excluded.version, based_on=excluded.based_on, \
             build=excluded.build, palette=excluded.palette, refs=excluded.refs",
            rusqlite::params![
                s.id,
                s.name,
                source.as_str().unwrap_or_default(),
                s.source_url,
                serde_json::to_string(&s.notes)?,
                s.prompt,
                s.created_at,
                engine,
                s.preview_url,
                s.preview_video_url,
                template,
                s.version,
                s.based_on,
                s.build,
                serde_json::to_string(&s.palette)?,
                serde_json::to_string(&s.references)?
            ],
        )?;
        Ok(())
    }

    pub fn delete_style(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        if Self::is_builtin(&conn, id)? {
            return Err(DbError("built-in styles cannot be deleted".into()));
        }
        conn.execute("DELETE FROM styles WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Tests: a style row gone the way older versions could lose it (a built-in dropped by sync).
    #[cfg(test)]
    pub fn remove_style_row(&self, id: &str) {
        self.conn().execute("DELETE FROM styles WHERE id = ?1", [id]).unwrap();
    }

    fn is_builtin(conn: &Connection, id: &str) -> Result<bool> {
        let source: Option<String> = conn
            .query_row("SELECT source FROM styles WHERE id = ?1", [id], |r| r.get(0))
            .optional()?;
        Ok(source.as_deref() == Some("builtin"))
    }

    // ---- jobs ----

    pub fn upsert_job(&self, j: &Job) -> Result<()> {
        let vals = bind_values(j, &JOB_KEYS)?;
        self.conn().execute(
            "INSERT INTO jobs (id, kind, title, project_id, chat_id, agent_id, status, progress, note, source_url, updated_at, render_baseline_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, \
             title=excluded.title, project_id=excluded.project_id, chat_id=excluded.chat_id, \
             agent_id=excluded.agent_id, status=excluded.status, progress=excluded.progress, \
             note=excluded.note, source_url=excluded.source_url, updated_at=excluded.updated_at, \
             render_baseline_at=excluded.render_baseline_at, \
             archived_at=CASE WHEN jobs.status = excluded.status THEN jobs.archived_at ELSE NULL END",
            params_from_iter(vals),
        )?;
        Ok(())
    }

    /// Marks jobs still `working` as failed with `note`: their agent died with the app.
    /// Returns how many changed.
    pub fn fail_stale_jobs(&self, note: &str) -> Result<usize> {
        Ok(self.conn().execute(
            // A permission request (needs-you, no note) died with its session too; a build turn that
            // stopped without a video (needs-you with its note) still waits for the user.
            "UPDATE jobs SET status = 'failed', note = ?1, updated_at = ?2 \
             WHERE status = 'working' OR (status = 'needs-you' AND note IS NULL)",
            rusqlite::params![note, now()],
        )?)
    }

    /// Removes a job from the board. A job that is working is never deleted (the board may be
    /// stale), and deleting a job that isn't there is fine.
    pub fn delete_job(&self, id: &str) -> Result<()> {
        self.conn().execute("DELETE FROM jobs WHERE id = ?1 AND status <> 'working'", [id])?;
        Ok(())
    }

    pub fn list_jobs(&self) -> Result<Vec<Job>> {
        let sql = format!("{JOB_SELECT} ORDER BY updated_at DESC, rowid DESC");
        query_as(&self.conn(), &sql, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn db() -> Db {
        Db::open_in_memory().unwrap()
    }

    fn project(id: &str, updated: &str) -> Project {
        Project {
            id: id.into(),
            title: format!("Title {id}"),
            slug: id.into(),
            folder: format!("/w/{id}"),
            kind: VideoKind::Promo,
            format: VideoFormat::Portrait,
            language: VideoLanguage::Id,
            status: ProjectStatus::Draft,
            style_id: None,
            duration_sec: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: updated.into(),
        }
    }

    #[test]
    fn settings_defaults_and_update() {
        let db = db();
        let home = PathBuf::from("/home/u");
        let s = db.get_settings(&home);
        assert_eq!(s.working_folder, home.join("Listune").to_string_lossy());
        assert_eq!(s.default_agent, AgentId::Claude);
        assert!(s.frame_check && !s.onboarded);
        assert_eq!((s.language, s.format, s.fps), (VideoLanguage::Id, VideoFormat::Portrait, 30));
        assert_eq!(s.ui_language, None);
        assert!(!s.background_mode && !s.open_at_login);
        let b = db.update_settings(json!({"backgroundMode": true, "openAtLogin": true})).unwrap();
        assert!(b.background_mode && b.open_at_login);
        assert!(db.get_settings(&home).background_mode);
        db.update_settings(json!({"backgroundMode": false, "openAtLogin": false})).unwrap();
        assert!(!s.build_jobs_checked, "the one-time build job check hasn't run");
        db.update_settings(json!({"buildJobsChecked": true})).unwrap();
        assert!(db.get_settings(&home).build_jobs_checked);
        assert!(!s.project_status_checked, "the one-time project status backfill hasn't run");
        db.update_settings(json!({"projectStatusChecked": true})).unwrap();
        assert!(db.get_settings(&home).project_status_checked);
        let l = db.update_settings(json!({"uiLanguage": "id"})).unwrap();
        assert_eq!(l.ui_language, Some(UiLanguage::Id));
        assert_eq!(db.get_settings(&home).ui_language, Some(UiLanguage::Id));

        let u = db.update_settings(json!({"fps": 60, "onboarded": true, "defaultAgent": "antigravity"})).unwrap();
        assert_eq!(u.fps, 60);
        let again = db.get_settings(&home);
        assert_eq!(again, u);
        assert!(again.onboarded);
        assert_eq!(again.default_agent, AgentId::Antigravity);
        assert_eq!(again.working_folder, home.join("Listune").to_string_lossy());
        assert!(db.update_settings(json!({"fps": "fast"})).is_err());
        assert!(db.update_settings(json!([1])).is_err());
        assert!(db.update_settings(json!("x")).is_err());
        assert_eq!(db.get_settings(&home).fps, 60);
    }

    #[test]
    fn update_settings_before_get_keeps_home_default() {
        let db = db();
        db.update_settings(json!({"fps": 24})).unwrap();
        let home = PathBuf::from("/home/u");
        let s = db.get_settings(&home);
        assert_eq!(s.fps, 24);
        assert_eq!(s.working_folder, home.join("Listune").to_string_lossy());
        let other = PathBuf::from("/home/v");
        assert_eq!(db.get_settings(&other).working_folder, other.join("Listune").to_string_lossy());
    }

    #[test]
    fn project_insert_list_update() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        db.insert_project(&project("b", "2026-02-01T00:00:00Z")).unwrap();
        let ids: Vec<_> = db.list_projects().unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["b", "a"]);
        assert_eq!(db.get_project("zzz").unwrap(), None);

        let up = db
            .update_project("a", json!({"title": "New", "status": "rendered", "durationSec": 12.5, "styleId": "builtin-news-recap"}))
            .unwrap();
        assert_eq!(up.title, "New");
        assert_eq!(up.status, ProjectStatus::Rendered);
        assert_eq!(up.duration_sec, Some(12.5));
        assert_eq!(db.get_project("a").unwrap().unwrap(), up);
        let ids: Vec<_> = db.list_projects().unwrap().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["a", "b"]);

        assert!(db.update_project("a", json!({"id": "other"})).is_err());
        assert!(db.update_project("a", json!({"status": "bogus"})).is_err());
        assert!(db.update_project("missing", json!({"title": "x"})).is_err());
        assert_eq!(db.get_project("a").unwrap().unwrap().title, "New");
    }

    #[test]
    fn project_status_only_moves_forward_unless_reset() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        let b = db.set_project_status("a", ProjectStatus::Building, false).unwrap();
        assert_eq!(b.status, ProjectStatus::Building);
        // A status move is not an edit: the project keeps its place in "recent" lists.
        assert_eq!(b.updated_at, "2026-01-01T00:00:00Z");
        assert_eq!(db.get_project("a").unwrap().unwrap().updated_at, "2026-01-01T00:00:00Z");
        // The same status again changes nothing, not even the time.
        let again = db.set_project_status("a", ProjectStatus::Building, false).unwrap();
        assert_eq!(again, b);
        let r = db.set_project_status("a", ProjectStatus::Rendered, false).unwrap();
        assert_eq!(r.status, ProjectStatus::Rendered);
        assert_eq!(r.updated_at, "2026-01-01T00:00:00Z");
        assert_eq!(db.set_project_status("a", ProjectStatus::Draft, true).unwrap().updated_at, "2026-01-01T00:00:00Z");
        db.set_project_status("a", ProjectStatus::Rendered, false).unwrap();
        // Never backwards on its own.
        assert_eq!(db.set_project_status("a", ProjectStatus::Building, false).unwrap().status, ProjectStatus::Rendered);
        assert_eq!(db.set_project_status("a", ProjectStatus::Draft, false).unwrap().status, ProjectStatus::Rendered);
        // An explicit reset may.
        assert_eq!(db.set_project_status("a", ProjectStatus::Draft, true).unwrap().status, ProjectStatus::Draft);
        assert_eq!(db.get_project("a").unwrap().unwrap().status, ProjectStatus::Draft);
        assert!(db.set_project_status("missing", ProjectStatus::Building, false).is_err());
    }

    #[test]
    fn set_chat_agent_persists_and_misses_unknown_chats() {
        let db = Db::open_in_memory().unwrap();
        let chat = db.create_chat(Some("a"), AgentId::Claude, "Fix intro").unwrap();
        let updated = db.set_chat_agent(&chat.id, AgentId::Codex).unwrap().unwrap();
        assert_eq!(updated.agent_id, AgentId::Codex);
        assert_eq!(updated.title, "Fix intro");
        assert_eq!(db.get_chat(&chat.id).unwrap().unwrap().agent_id, AgentId::Codex);
        assert_eq!(db.set_chat_agent("nope", AgentId::Antigravity).unwrap(), None);
    }

    #[test]
    fn an_archived_thread_leaves_recent_until_restored_or_written_in() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        let first = db.create_chat(Some("a"), AgentId::Claude, "First").unwrap();
        let second = db.create_chat(Some("a"), AgentId::Claude, "Second").unwrap();
        let loose = db.create_chat(None, AgentId::Claude, "Loose").unwrap();
        // Archiving one chat of a project archives the whole thread.
        db.set_thread_archived(&first.id, true).unwrap();
        let ids = |v: Vec<RecentChat>| v.into_iter().map(|c| c.id).collect::<Vec<_>>();
        assert_eq!(ids(db.list_recent_chats(None).unwrap()), vec![loose.id.clone()]);
        let archived = db.list_archived_chats().unwrap();
        assert_eq!(archived.len(), 1, "a project shows once");
        assert_eq!(archived[0].project_id.as_deref(), Some("a"));
        assert!(archived[0].archived_at.is_some());
        db.set_thread_archived(&second.id, false).unwrap();
        assert_eq!(db.list_recent_chats(None).unwrap().len(), 2);
        assert!(db.list_archived_chats().unwrap().is_empty());
        // The user writing in an archived thread brings it back; the agent writing does not.
        db.set_thread_archived(&loose.id, true).unwrap();
        db.add_message(&loose.id, MessageRole::Agent, &[]).unwrap();
        assert_eq!(db.list_archived_chats().unwrap().len(), 1);
        db.add_message(&loose.id, MessageRole::User, &[]).unwrap();
        assert!(db.list_archived_chats().unwrap().is_empty());
    }

    #[test]
    fn purging_the_archive_deletes_archived_chats_and_cards_only() {
        let db = db();
        let gone = db.create_chat(None, AgentId::Claude, "Gone").unwrap();
        let kept = db.create_chat(None, AgentId::Claude, "Kept").unwrap();
        db.add_message(&gone.id, MessageRole::User, &[]).unwrap();
        db.add_message(&kept.id, MessageRole::User, &[]).unwrap();
        db.set_thread_archived(&gone.id, true).unwrap();
        let job = |id: &str, status: JobStatus| Job {
            id: id.into(),
            kind: JobKind::Build,
            title: id.into(),
            project_id: None,
            chat_id: None,
            agent_id: AgentId::Claude,
            status,
            progress: None,
            note: None,
            source_url: None,
            render_baseline_at: None,
            updated_at: "2026-01-01T00:00:00Z".into(),
            archived_at: None,
        };
        db.upsert_job(&job("done", JobStatus::Done)).unwrap();
        db.upsert_job(&job("busy", JobStatus::Working)).unwrap();
        db.upsert_job(&job("other", JobStatus::Done)).unwrap();
        db.set_jobs_archived(&["done".into(), "busy".into()], true).unwrap();
        let archived = |id: &str| db.list_jobs().unwrap().into_iter().find(|j| j.id == id).unwrap().archived_at.is_some();
        assert!(archived("done"));
        assert!(!archived("busy"), "a working card is never archived");
        // Rewriting a card with the same status keeps it archived; a new status brings it back.
        db.upsert_job(&job("done", JobStatus::Done)).unwrap();
        assert!(archived("done"));
        db.upsert_job(&job("other", JobStatus::Done)).unwrap();
        db.set_jobs_archived(&["other".into()], true).unwrap();
        db.upsert_job(&job("other", JobStatus::Working)).unwrap();
        assert!(!archived("other"));

        assert_eq!(db.purge_archive().unwrap(), (1, 1));
        assert!(db.get_chat(&gone.id).unwrap().is_none());
        assert!(db.list_messages(&gone.id).unwrap().is_empty());
        assert_eq!(db.list_messages(&kept.id).unwrap().len(), 1);
        let left: Vec<String> = db.list_jobs().unwrap().into_iter().map(|j| j.id).collect();
        assert!(!left.contains(&"done".to_string()) && left.contains(&"busy".to_string()));
    }

    #[test]
    fn recent_chats_newest_activity_first_with_project_titles() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        let video = db.create_chat(Some("a"), AgentId::Claude, "Video chat").unwrap();
        let loose = db.create_chat(None, AgentId::Codex, "Loose").unwrap();
        let quiet = db.create_chat(None, AgentId::Claude, "Quiet").unwrap();
        // A chat of a deleted project has nowhere to go: it is left out.
        let orphan = db.create_chat(Some("gone"), AgentId::Claude, "Orphan").unwrap();
        let conn = db.conn();
        conn.execute("UPDATE chats SET created_at = '2026-01-01T00:00:00Z'", []).unwrap();
        for (chat, at) in [(&video, "2026-01-02T00:00:00Z"), (&loose, "2026-01-03T00:00:00Z"), (&orphan, "2026-01-09T00:00:00Z")] {
            conn.execute(
                "INSERT INTO messages (id, chat_id, role, parts, created_at) VALUES (?1, ?2, 'user', '[]', ?3)",
                rusqlite::params![format!("m-{}", chat.id), chat.id, at],
            )
            .unwrap();
        }
        drop(conn);

        let recent = db.list_recent_chats(None).unwrap();
        let ids: Vec<&str> = recent.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec![loose.id.as_str(), video.id.as_str(), quiet.id.as_str()]);
        assert_eq!(recent[0].project_title, None);
        assert_eq!(recent[0].active_at, "2026-01-03T00:00:00Z");
        assert_eq!(recent[1].project_title.as_deref(), Some("Title a"));
        assert_eq!(recent[1].project_id.as_deref(), Some("a"));
        // No messages yet: the chat counts from when it was made.
        assert_eq!(recent[2].active_at, "2026-01-01T00:00:00Z");
        assert_eq!(recent[2].title, "Quiet");

        let top = db.list_recent_chats(Some(1)).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, loose.id);
    }

    #[test]
    fn messages_have_a_chat_and_time_index_that_survives_reopening() {
        let db = db();
        let names = |db: &Db| -> Vec<String> {
            let conn = db.conn();
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = 'messages' ORDER BY name")
                .unwrap();
            stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
        };
        assert!(names(&db).contains(&"messages_chat_at".to_string()));
        // Running the schema again (every start) changes nothing.
        db.conn().execute_batch(SCHEMA).unwrap();
        assert_eq!(names(&db).iter().filter(|n| n.as_str() == "messages_chat_at").count(), 1);
        let conn = db.conn();
        let plan: String = conn
            .query_row(
                "EXPLAIN QUERY PLAN SELECT MAX(created_at) FROM messages WHERE chat_id = 'x'",
                [],
                |r| r.get(3),
            )
            .unwrap();
        assert!(plan.contains("messages_chat_at"), "{plan}");
    }

    #[test]
    fn recent_chats_count_a_project_once_before_the_limit() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        let busy: Vec<Chat> = (0..5).map(|i| db.create_chat(Some("a"), AgentId::Claude, &format!("A{i}")).unwrap()).collect();
        let loose = db.create_chat(None, AgentId::Claude, "Loose").unwrap();
        let conn = db.conn();
        conn.execute("UPDATE chats SET created_at = '2026-01-01T00:00:00Z'", []).unwrap();
        conn.execute("UPDATE chats SET created_at = '2026-01-02T00:00:00Z' WHERE id = ?1", [&busy[2].id]).unwrap();
        conn.execute("UPDATE chats SET created_at = '2026-01-03T00:00:00Z' WHERE project_id = 'a' AND id != ?1", [&busy[2].id])
            .unwrap();
        drop(conn);
        let recent = db.list_recent_chats(Some(2)).unwrap();
        let ids: Vec<&str> = recent.iter().map(|c| c.id.as_str()).collect();
        // The project's most active chat stands for it; the limit counts distinct entries.
        assert_eq!(ids.len(), 2);
        assert_eq!(recent[0].project_id.as_deref(), Some("a"));
        assert_ne!(recent[0].id, busy[2].id);
        assert_eq!(ids[1], loose.id.as_str());
    }

    #[test]
    fn chat_and_messages_round_trip() {
        let db = db();
        db.insert_project(&project("a", "2026-01-01T00:00:00Z")).unwrap();
        let chat = db.create_chat(Some("a"), AgentId::Claude, "Fix intro").unwrap();
        let global = db.create_chat(None, AgentId::Codex, "General").unwrap();
        assert_eq!(db.list_chats(Some("a")).unwrap(), vec![chat.clone()]);
        assert_eq!(db.list_chats(None).unwrap(), vec![global]);
        assert_eq!(db.get_chat(&chat.id).unwrap(), Some(chat.clone()));
        assert_eq!(db.get_chat("nope").unwrap(), None);

        let frame = MessagePart::Frame {
            project_id: "a".into(),
            frame: 42,
            time_sec: 1.4,
            image_path: "/x.png".into(),
            marks: vec![Mark { x: 0.5, y: 0.25 }],
        };
        let m1 = db.add_message(&chat.id, MessageRole::User, &[MessagePart::Text { text: "look".into() }, frame.clone()]).unwrap();
        let m2 = db.add_message(&chat.id, MessageRole::Agent, &[MessagePart::Text { text: "ok".into() }]).unwrap();
        let got = db.list_messages(&chat.id).unwrap();
        assert_eq!(got, vec![m1, m2]);
        assert_eq!(got[0].parts[1], frame);
    }

    fn builtin(id: &str, template: bool) -> Style {
        Style {
            id: id.into(),
            name: id.into(),
            source: StyleSource::Builtin,
            source_url: None,
            notes: BTreeMap::new(),
            prompt: "p".into(),
            created_at: "2026-11-04T00:00:00Z".into(),
            engine: Some(StyleEngine::Remotion),
            preview_url: Some(format!("https://api/v1/styles/{id}/preview")),
            preview_video_url: Some(format!("https://api/v1/styles/{id}/preview-video")),
            based_on: None,
            template: template
                .then(|| StyleTemplate { sha256: "ab".repeat(32), size: 10, version: 2 }),
            version: Some(2),
            build: None,
            palette: vec![],
            references: vec![],
        }
    }

    #[test]
    fn a_copied_style_keeps_its_recipe_palette_and_references() {
        let db = db();
        let copied = Style {
            source: StyleSource::Copied,
            source_url: Some("https://youtu.be/x".into()),
            template: None,
            preview_url: None,
            preview_video_url: None,
            build: Some("three.js in Remotion".into()),
            palette: vec!["#5BC8F5".into(), "#F2C14E".into()],
            references: vec!["/d/style-refs/copied-1/ref-1.jpg".into()],
            ..builtin("copied-1", false)
        };
        db.save_style(&copied).unwrap();
        assert_eq!(db.get_style("copied-1").unwrap(), Some(copied));
    }

    #[test]
    fn seed_builtin_styles_populates_all_28_styles_with_previews_and_templates() {
        let db = db();
        assert!(db.list_styles().unwrap().is_empty());
        db.seed_builtin_styles().unwrap();
        let styles = db.list_styles().unwrap();
        assert_eq!(styles.len(), 28);

        // No duplicate IDs
        let unique_ids: std::collections::HashSet<&str> = styles.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(unique_ids.len(), 28);

        for s in &styles {
            assert_eq!(s.source, StyleSource::Builtin);
            assert!(!s.id.is_empty());
            assert!(!s.name.is_empty());
            assert!(!s.prompt.is_empty());
            assert!(s.engine.is_some());
            assert!(s.preview_url.as_deref().is_some_and(|u| !u.trim().is_empty()), "style {} must have preview_url", s.id);
            assert!(s.preview_video_url.as_deref().is_some_and(|u| !u.trim().is_empty()), "style {} must have preview_video_url", s.id);
            assert!(s.notes.contains_key(&StyleAspect::Camera));
            assert!(s.notes.contains_key(&StyleAspect::Pacing));
            assert!(s.notes.contains_key(&StyleAspect::Type));
            assert!(s.notes.contains_key(&StyleAspect::Color));
            assert!(s.notes.contains_key(&StyleAspect::Captions));
            assert!(s.notes.contains_key(&StyleAspect::Transitions));
            if s.source_url.as_deref() == Some("https://github.com/lemomo-ai/lemo-opuscar") {
                assert!(!s.palette.is_empty(), "Opuscar style {} must have palette", s.id);
            }
        }

        // Check the 3 original styles are present with previews and Remotion templates
        for id in ["atlas-journey", "cartoon-3d-world", "cartoon-3d-what-if"] {
            let s = db.get_style(id).unwrap().expect("original style must exist");
            assert_eq!(s.engine, Some(StyleEngine::Remotion));
            assert!(s.preview_url.as_deref().is_some_and(|u| u.contains(id)), "{id} preview_url");
            assert!(s.preview_video_url.as_deref().is_some_and(|u| u.contains(id)), "{id} preview_video_url");
            let t = s.template.expect("original template must be defined");
            assert!(!t.sha256.is_empty());
            assert!(t.size > 0);
        }

        // Check the 10 overlapping styles retain their previewUrl and previewVideoUrl
        for id in [
            "whiteboard", "dataviz", "iso-infographic", "transit-map", "chalkboard",
            "swiss-motion", "watercolor", "shadow-puppet", "papercut-red", "midcentury-toon",
        ] {
            let s = db.get_style(id).unwrap().expect("overlapping style must exist");
            assert!(s.preview_url.as_deref().is_some_and(|u| u.contains(id)), "{id} missing preview_url");
            assert!(s.preview_video_url.as_deref().is_some_and(|u| u.contains(id)), "{id} missing preview_video_url");
        }

        // Check 4 active starter templates
        for id in ["swiss-motion", "dataviz", "crayon-book", "satellite-briefing"] {
            let s = db.get_style(id).unwrap().expect("starter template style must exist");
            let t = s.template.expect("template must be defined");
            assert!(!t.sha256.is_empty());
            assert!(t.size > 0);
            assert_eq!(t.version, 1);
        }

        // Re-seeding is idempotent
        db.seed_builtin_styles().unwrap();
        assert_eq!(db.list_styles().unwrap().len(), 28);
        assert!(db.delete_style("swiss-motion").is_err(), "built-ins stay read-only");
    }

    #[test]
    fn seed_builtin_styles_restores_all_28_over_partial_legacy_db_without_duplicates() {
        let db = db();
        // Simulate an older DB that only had 13 styles
        let initial_13 = [
            "atlas-journey",
            "cartoon-3d-world",
            "cartoon-3d-what-if",
            "whiteboard",
            "dataviz",
            "iso-infographic",
            "transit-map",
            "chalkboard",
            "swiss-motion",
            "watercolor",
            "shadow-puppet",
            "papercut-red",
            "midcentury-toon",
        ]
        .into_iter()
        .map(|id| Style {
            id: id.into(),
            name: id.into(),
            source: StyleSource::Builtin,
            source_url: None,
            notes: BTreeMap::new(),
            prompt: "existing prompt".into(),
            created_at: "2026-11-09T00:00:00Z".into(),
            engine: Some(StyleEngine::Hyperframes),
            preview_url: Some(format!("https://example.test/styles/{id}/preview.jpg")),
            preview_video_url: Some(format!("https://example.test/styles/{id}/preview.mp4")),
            based_on: None,
            template: None,
            version: Some(1),
            build: None,
            palette: vec![],
            references: vec![],
        })
        .collect::<Vec<_>>();
        db.seed_rows_for_test(&initial_13).unwrap();
        assert_eq!(db.list_styles().unwrap().len(), 13);

        // Now seed_builtin_styles adds the remaining 15 active Opuscar styles while keeping all 13 + restoring templates
        db.seed_builtin_styles().unwrap();
        let all = db.list_styles().unwrap();
        assert_eq!(all.len(), 28);
        let unique: std::collections::HashSet<&str> = all.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(unique.len(), 28);
        for id in ["atlas-journey", "cartoon-3d-world", "cartoon-3d-what-if", "swiss-motion", "dataviz"] {
            assert!(db.get_style(id).unwrap().unwrap().template.is_some(), "{id} should have template");
            assert_eq!(db.get_style(id).unwrap().unwrap().engine, Some(StyleEngine::Remotion));
        }
    }

    #[test]
    fn user_styles_work_without_seeding() {
        let db = db();
        assert!(db.list_styles().unwrap().is_empty());
        let mine = Style {
            source: StyleSource::Mine,
            engine: None,
            preview_url: None,
            preview_video_url: None,
            template: None,
            based_on: None,
            version: None,
            ..builtin("s1", false)
        };
        db.save_style(&mine).unwrap();
        assert_eq!(db.list_styles().unwrap(), vec![mine.clone()]);
        db.delete_style("s1").unwrap();
        assert!(db.list_styles().unwrap().is_empty());
    }

    #[test]
    fn a_clone_keeps_template_previews_and_based_on() {
        let db = db();
        db.seed_builtin_styles().unwrap();
        let swiss = db.get_style("swiss-motion").unwrap().unwrap();
        let clone = Style {
            id: "mine-1".into(),
            name: "Swiss Motion (copy)".into(),
            source: StyleSource::Mine,
            based_on: Some("swiss-motion".into()),
            ..swiss
        };
        db.save_style(&clone).unwrap();
        assert_eq!(db.get_style("mine-1").unwrap().unwrap(), clone);
        let edited = Style { name: "Mine".into(), ..clone.clone() };
        db.save_style(&edited).unwrap();
        db.seed_builtin_styles().unwrap();
        assert_eq!(db.get_style("mine-1").unwrap().unwrap(), edited);
        assert!(db.get_style("swiss-motion").unwrap().is_some());
    }

    #[test]
    fn seed_builtin_styles_never_overwrites_a_row_that_is_not_built_in() {
        let db = db();
        let mine = Style {
            id: "swiss-motion".into(),
            name: "Mine".into(),
            source: StyleSource::Mine,
            ..builtin("swiss-motion", false)
        };
        db.save_style(&mine).unwrap();
        db.seed_builtin_styles().unwrap();
        assert_eq!(db.get_style("swiss-motion").unwrap().unwrap(), mine);
    }

    #[test]
    fn migration_adds_columns_and_drops_legacy_builtins() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("listune.db");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch(
                "CREATE TABLE styles (id TEXT PRIMARY KEY, name TEXT NOT NULL, source TEXT NOT NULL, \
                 source_url TEXT, notes TEXT NOT NULL, prompt TEXT NOT NULL, created_at TEXT NOT NULL);
                 INSERT INTO styles VALUES ('builtin-bold-explainer','x','builtin',NULL,'{}','p','t');
                 INSERT INTO styles VALUES ('keep','y','mine',NULL,'{}','p','t');",
            )
            .unwrap();
        }
        let db = Db::open(&path).unwrap();
        let styles = db.list_styles().unwrap();
        // 28 bundled styles (3 original + 25 active Opuscar) seeded on Db::open + 1 kept mine style = 29
        assert_eq!(styles.len(), 29);
        assert!(styles.iter().any(|s| s.id == "keep"));
        assert!(!styles.iter().any(|s| s.id == "builtin-bold-explainer"));
        let keep = db.get_style("keep").unwrap().unwrap();
        assert_eq!(keep.template, None);
        assert_eq!(keep.preview_video_url, None);
        assert_eq!(keep.based_on, None);
        drop(db);
        assert_eq!(Db::open(&path).unwrap().list_styles().unwrap().len(), 29);
    }

    #[test]
    fn seed_builtin_styles_preserves_existing_custom_fields() {
        let db = db();
        db.seed_builtin_styles().unwrap();

        let custom_whiteboard = Style {
            id: "whiteboard".into(),
            name: "Whiteboard explainer".into(),
            source: StyleSource::Builtin,
            source_url: None,
            notes: BTreeMap::from([(StyleAspect::Camera, "Custom camera note".into())]),
            prompt: "Full 11k Listune-adapted HyperFrames guide...".into(),
            created_at: "2026-11-10T00:00:00Z".into(),
            engine: Some(StyleEngine::Hyperframes),
            preview_url: Some("https://example.test/styles/whiteboard/preview.jpg".into()),
            preview_video_url: Some("https://example.test/styles/whiteboard/preview.mp4".into()),
            based_on: None,
            template: None,
            version: Some(3),
            build: None,
            palette: vec![],
            references: vec![],
        };
        let custom_atlas = Style {
            id: "atlas-journey".into(),
            name: "Atlas journey v9".into(),
            source: StyleSource::Builtin,
            source_url: None,
            notes: BTreeMap::new(),
            prompt: "Updated Atlas prompt v9".into(),
            created_at: "2026-11-10T00:00:00Z".into(),
            engine: Some(StyleEngine::Remotion),
            preview_url: Some("https://example.test/styles/atlas-journey/preview.jpg".into()),
            preview_video_url: Some("https://example.test/styles/atlas-journey/preview.mp4".into()),
            based_on: None,
            template: Some(StyleTemplate { sha256: "ff".repeat(32), size: 999, version: 9 }),
            version: Some(9),
            build: None,
            palette: vec![],
            references: vec![],
        };
        db.seed_rows_for_test(&[custom_whiteboard, custom_atlas]).unwrap();

        // Subsequent seed_builtin_styles() (e.g. from commands::list_styles or restart) must NOT clobber existing fields.
        db.seed_builtin_styles().unwrap();
        let wb = db.get_style("whiteboard").unwrap().unwrap();
        assert_eq!(wb.name, "Whiteboard explainer");
        assert_eq!(wb.engine, Some(StyleEngine::Hyperframes));
        assert_eq!(wb.prompt, "Full 11k Listune-adapted HyperFrames guide...");
        assert_eq!(wb.preview_url.as_deref(), Some("https://example.test/styles/whiteboard/preview.jpg"));
        assert_eq!(wb.notes.get(&StyleAspect::Camera).map(String::as_str), Some("Custom camera note"));
        assert!(!wb.palette.is_empty());
        assert!(wb.build.is_some());

        let atlas = db.get_style("atlas-journey").unwrap().unwrap();
        assert_eq!(atlas.name, "Atlas journey v9");
        assert_eq!(atlas.version, Some(9));
        assert_eq!(atlas.prompt, "Updated Atlas prompt v9");
        assert_eq!(atlas.preview_url.as_deref(), Some("https://example.test/styles/atlas-journey/preview.jpg"));
        assert_eq!(atlas.template.unwrap().version, 9);
    }

    #[test]
    fn seed_builtin_styles_restores_missing_3_initial_styles_and_null_previews_over_25_opuscar_only_db() {
        let db = db();
        let opuscar_only: Vec<Style> = bundled_builtin_styles()
            .iter()
            .filter(|s| !matches!(s.id.as_str(), "atlas-journey" | "cartoon-3d-world" | "cartoon-3d-what-if"))
            .map(|s| Style {
                preview_url: None,
                preview_video_url: None,
                ..s.clone()
            })
            .collect();
        assert_eq!(opuscar_only.len(), 25);
        db.seed_rows_for_test(&opuscar_only).unwrap();
        assert_eq!(db.list_styles().unwrap().len(), 25);

        db.seed_builtin_styles().unwrap();
        let all = db.list_styles().unwrap();
        assert_eq!(all.len(), 28);
        assert_eq!(all.iter().filter(|s| s.preview_url.is_some() && s.preview_video_url.is_some()).count(), 28);
        assert_eq!(all.iter().filter(|s| s.template.is_some()).count(), 7);
        for id in ["atlas-journey", "cartoon-3d-world", "cartoon-3d-what-if", "whiteboard", "swiss-motion", "archive-lighttable"] {
            assert!(db.get_style(id).unwrap().unwrap().preview_url.is_some(), "{id} preview_url restored");
        }
    }

    #[test]
    fn seed_builtin_styles_prunes_dropped_styles_unless_referenced() {
        let db = db();
        db.seed_rows_for_test(&[builtin("blueprint", false), builtin("ukiyoe", false)]).unwrap();
        let clone = Style {
            id: "mine-ukiyoe".into(),
            name: "Ukiyoe Clone".into(),
            source: StyleSource::Mine,
            based_on: Some("ukiyoe".into()),
            ..builtin("ukiyoe", false)
        };
        db.save_style(&clone).unwrap();

        db.seed_builtin_styles().unwrap();
        assert!(db.get_style("blueprint").unwrap().is_none(), "unused dropped style must be pruned");
        assert!(db.get_style("ukiyoe").unwrap().is_some(), "dropped style referenced by clone stays safe");
        assert!(db.get_style("atlas-journey").unwrap().is_some());
        assert!(db.get_style("archive-lighttable").unwrap().is_some());
    }

    #[test]
    fn jobs_upsert_and_list() {
        let db = db();
        let mut j = Job {
            id: "j1".into(),
            kind: JobKind::Render,
            title: "Render".into(),
            project_id: None,
            chat_id: None,
            agent_id: AgentId::Claude,
            status: JobStatus::Queued,
            progress: None,
            note: None,
            source_url: None,
            render_baseline_at: None,
            archived_at: None,
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        db.upsert_job(&j).unwrap();
        j.status = JobStatus::Working;
        j.progress = Some(0.5);
        j.updated_at = "2026-01-02T00:00:00Z".into();
        j.render_baseline_at = Some("2026-01-01T23:59:00.000Z".into());
        db.upsert_job(&j).unwrap();
        assert_eq!(db.list_jobs().unwrap(), vec![j.clone()]);
        // A job written without it (an older UI) has none.
        let raw = serde_json::json!({ "id": "j2", "kind": "build", "title": "B", "projectId": null, "chatId": null, "agentId": "claude",
            "status": "done", "progress": null, "note": null, "sourceUrl": null, "updatedAt": "2026-01-03T00:00:00Z" });
        db.upsert_job(&serde_json::from_value(raw).unwrap()).unwrap();
        assert_eq!(db.list_jobs().unwrap()[0].render_baseline_at, None);
    }

    #[test]
    fn jobs_migration_adds_source_url_to_an_old_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("listune.db");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch(
                "CREATE TABLE jobs (id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, \
                 project_id TEXT, chat_id TEXT, agent_id TEXT NOT NULL, status TEXT NOT NULL, \
                 progress REAL, note TEXT, updated_at TEXT NOT NULL);
                 INSERT INTO jobs VALUES ('old','study-style','Study','{}',NULL,'claude','failed',NULL,'n','t');",
            )
            .unwrap();
        }
        let db = Db::open(&path).unwrap();
        let jobs = db.list_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].source_url, None);
        assert_eq!(jobs[0].render_baseline_at, None);
        drop(db);
        // Opening again must not try to add the column twice.
        assert_eq!(Db::open(&path).unwrap().list_jobs().unwrap().len(), 1);
    }

    #[test]
    fn legacy_calendar_and_autopilot_schema_is_cleaned_up_safely() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("listune.db");
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch(
                "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 INSERT INTO settings VALUES ('authAccount:api:https://studio.listune.app/api', '{}');
                 CREATE TABLE projects (id TEXT PRIMARY KEY, title TEXT NOT NULL, slug TEXT NOT NULL, \
                 folder TEXT NOT NULL, kind TEXT NOT NULL, format TEXT NOT NULL, language TEXT NOT NULL, \
                 status TEXT NOT NULL, style_id TEXT, scheduled_at TEXT, duration_sec REAL, \
                 created_at TEXT NOT NULL, updated_at TEXT NOT NULL, \
                 review_state TEXT NOT NULL DEFAULT 'none', posted_at TEXT, reminded_at TEXT, review_reminded_at TEXT);
                 CREATE TABLE autopilots (id TEXT PRIMARY KEY, name TEXT NOT NULL);
                 CREATE TABLE autopilot_slots (id TEXT PRIMARY KEY, autopilot_id TEXT NOT NULL);
                 INSERT INTO projects VALUES ('old','T','t','/w/t','promo','9:16','id','scheduled',NULL,'2026-11-05T09:00:00Z',NULL,'c','u','pending',NULL,NULL,NULL);",
            )
            .unwrap();
        }
        let db = Db::open(&path).unwrap();
        let p = db.get_project("old").unwrap().unwrap();
        assert_eq!(p.status, ProjectStatus::Rendered);
        let conn = db.conn();
        for tbl in ["autopilots", "autopilot_slots"] {
            let exists: bool = conn
                .query_row("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1", [tbl], |_| Ok(true))
                .optional()
                .unwrap()
                .unwrap_or(false);
            assert!(!exists, "{tbl} should be dropped");
        }
        let auth_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings WHERE key LIKE 'authAccount:%'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(auth_rows, 0);
    }

    #[test]
    fn delete_job_removes_only_that_job() {
        let db = db();
        let job = |id: &str| Job {
            id: id.into(),
            kind: JobKind::Build,
            title: id.into(),
            project_id: None,
            chat_id: None,
            agent_id: AgentId::Claude,
            status: JobStatus::Failed,
            progress: None,
            note: None,
            source_url: None,
            render_baseline_at: None,
            archived_at: None,
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        db.upsert_job(&job("a")).unwrap();
        db.upsert_job(&job("b")).unwrap();
        let mut working = job("w");
        working.status = JobStatus::Working;
        db.upsert_job(&working).unwrap();
        db.delete_job("a").unwrap();
        db.delete_job("missing").unwrap();
        // A job that is working again (stale board) is never deleted.
        db.delete_job("w").unwrap();
        let ids: Vec<_> = db.list_jobs().unwrap().into_iter().map(|j| j.id).collect();
        assert!(ids.contains(&"w".to_string()) && ids.contains(&"b".to_string()) && !ids.contains(&"a".to_string()));
    }

    #[test]
    fn stale_working_jobs_fail_on_start() {
        let db = db();
        let job = |id: &str, status: JobStatus| Job {
            id: id.into(),
            kind: JobKind::Build,
            title: id.into(),
            project_id: None,
            chat_id: None,
            agent_id: AgentId::Claude,
            status,
            progress: None,
            note: None,
            source_url: None,
            render_baseline_at: None,
            archived_at: None,
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let mut w = job("w", JobStatus::Working);
        w.source_url = Some("https://youtu.be/abc".into());
        db.upsert_job(&w).unwrap();
        db.upsert_job(&job("d", JobStatus::Done)).unwrap();
        db.upsert_job(&job("p", JobStatus::NeedsYou)).unwrap();
        let mut stopped = job("s", JobStatus::NeedsYou);
        stopped.note = Some("No video yet. The agent stopped before rendering".into());
        db.upsert_job(&stopped).unwrap();
        assert_eq!(db.fail_stale_jobs("closed").unwrap(), 2);
        let after = db.list_jobs().unwrap();
        assert_eq!(after.iter().find(|j| j.id == "p").unwrap().status, JobStatus::Failed);
        assert_eq!(after.iter().find(|j| j.id == "s").unwrap().status, JobStatus::NeedsYou);
        let jobs = db.list_jobs().unwrap();
        let w = jobs.iter().find(|j| j.id == "w").unwrap();
        assert_eq!((w.status, w.note.as_deref()), (JobStatus::Failed, Some("closed")));
        assert_eq!(w.source_url.as_deref(), Some("https://youtu.be/abc"));
        assert_eq!(jobs.iter().find(|j| j.id == "d").unwrap().status, JobStatus::Done);
    }
}
