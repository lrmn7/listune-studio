//! One-time migration from Listune Studio's local app data folder (`app.Listune.studio` or
//! `app.Listune.studio.dev`, containing `Listune.db`) to Listune Studio (`listune.db`). Runs before
//! `Db::open` so existing projects, chats, custom/copied styles and media caches carry forward
//! intact.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

const LEGACY_APP_ID: &str = "app.tayang.studio";
const NEW_APP_ID: &str = "app.listune.studio";
const LEGACY_CLOSED_NOTE: &str = "Tayang was closed while this was running";
const NEW_CLOSED_NOTE: &str = "Listune was closed while this was running";
const DATA_SUBDIRS: [&str; 5] = ["style-media", "style-refs", "thumbs", "templates", "claude"];

/// Returns the sibling Listune Studio data directory for a Listune Studio data directory, if the
/// directory name matches `app.listune.studio` (including `.dev`).
pub fn legacy_data_dir(data: &Path) -> Option<PathBuf> {
    let name = data.file_name()?.to_str()?;
    let suffix = name.strip_prefix(NEW_APP_ID)?;
    Some(data.parent()?.join(format!("{LEGACY_APP_ID}{suffix}")))
}

fn pct_encode_bytes(input: &str, uppercase_hex: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.as_bytes() {
        let keep = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/');
        if keep {
            out.push(*b as char);
        } else if uppercase_hex {
            out.push_str(&format!("%{:02X}", b));
        } else {
            out.push_str(&format!("%{:02x}", b));
        }
    }
    out
}

/// Rewrites local paths and `asset.localhost` / `asset://` URLs pointing into `old_dir` so they
/// point into `new_dir` instead. Never touches remote `https://studio.listune.app/...` URLs.
pub fn rewrite_local_ref(value: &str, old_dir: &Path, new_dir: &Path) -> String {
    if old_dir == new_dir {
        return value.to_string();
    }
    let old_raw = old_dir.to_string_lossy();
    let new_raw = new_dir.to_string_lossy();
    let old_fwd = old_raw.replace('\\', "/");
    let new_fwd = new_raw.replace('\\', "/");
    let old_json = old_raw.replace('\\', "\\\\");
    let new_json = new_raw.replace('\\', "\\\\");

    let old_pct_upper = pct_encode_bytes(&old_raw, true);
    let old_pct_lower = pct_encode_bytes(&old_raw, false);
    let new_pct_upper = pct_encode_bytes(&new_raw, true);
    let old_fwd_pct_upper = pct_encode_bytes(&old_fwd, true);
    let old_fwd_pct_lower = pct_encode_bytes(&old_fwd, false);
    let new_fwd_pct_upper = pct_encode_bytes(&new_fwd, true);

    let mut out = value
        .replace(old_json.as_str(), &new_json)
        .replace(old_raw.as_ref(), &new_raw)
        .replace(&old_fwd, &new_fwd)
        .replace(&old_pct_upper, &new_pct_upper)
        .replace(&old_pct_lower, &new_pct_upper)
        .replace(&old_fwd_pct_upper, &new_fwd_pct_upper)
        .replace(&old_fwd_pct_lower, &new_fwd_pct_upper);

    // Also handle the case where `old_name` appears inside a local asset URL (`asset.localhost` /
    // `asset://`) with mixed path encoding. Never rewrite upstream `studio.listune.app` URLs.
    if let (Some(old_name), Some(new_name)) = (
        old_dir.file_name().and_then(|s| s.to_str()),
        new_dir.file_name().and_then(|s| s.to_str()),
    ) {
        if out.contains("asset.localhost") || out.contains("asset://") {
            for (pat, rep) in [
                (format!("/{old_name}/"), format!("/{new_name}/")),
                (format!("%5C{old_name}%5C"), format!("%5C{new_name}%5C")),
                (format!("%5c{old_name}%5c"), format!("%5c{new_name}%5c")),
            ] {
                out = out.replace(&pat, &rep);
            }
        }
    }
    out
}

fn copy_dir_missing(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(dst).map_err(|e| format!("create {}: {e}", dst.display()))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("read {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ft = entry.file_type().map_err(|e| e.to_string())?;
        let target = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_missing(&entry.path(), &target)?;
        } else if ft.is_file() && !target.exists() {
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("copy {} -> {}: {e}", entry.path().display(), target.display()))?;
        }
    }
    Ok(())
}

fn copy_top_files(src: &Path, dst: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(src).map_err(|e| format!("read {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else { continue };
        let wanted = name_str == "agent-usage.jsonl";
        if wanted {
            let target = dst.join(&name);
            if !target.exists() {
                std::fs::copy(entry.path(), &target)
                    .map_err(|e| format!("copy {} -> {}: {e}", entry.path().display(), target.display()))?;
            }
        }
    }
    Ok(())
}

fn rewrite_migrated_db(conn: &Connection, src_dir: &Path, data: &Path) -> Result<(), String> {
    // Only rewrite tables if they exist (older schemas or minimal DBs stay safe).
    let has_jobs: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'jobs'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if has_jobs {
        conn.execute(
            "UPDATE jobs SET note = ?1 WHERE note = ?2",
            rusqlite::params![NEW_CLOSED_NOTE, LEGACY_CLOSED_NOTE],
        )
        .map_err(|e| e.to_string())?;
    }

    if src_dir != data {
        let has_styles: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'styles'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if has_styles {
            let mut col_stmt = conn.prepare("PRAGMA table_info(styles)").map_err(|e| e.to_string())?;
            let cols: Vec<String> = col_stmt
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            drop(col_stmt);

            for col in ["preview_url", "preview_video_url", "refs"] {
                if !cols.iter().any(|c| c == col) {
                    continue;
                }
                let select_sql = format!("SELECT id, {col} FROM styles WHERE {col} IS NOT NULL");
                let mut stmt = conn.prepare(&select_sql).map_err(|e| e.to_string())?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
                    .map_err(|e| e.to_string())?;
                let collected: Vec<_> = rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?;
                drop(stmt);

                let update_sql = format!("UPDATE styles SET {col} = ?1 WHERE id = ?2");
                for (id, val) in collected {
                    let next = rewrite_local_ref(&val, src_dir, data);
                    if next != val {
                        conn.execute(&update_sql, rusqlite::params![next, id])
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Copies and upgrades legacy Listune Studio local state into `data/listune.db` if `listune.db`
/// does not exist yet. Returns `Ok(true)` when migration ran, `Ok(false)` when skipped.
pub fn migrate_legacy_data(data: &Path) -> Result<bool, String> {
    let target_db = data.join("listune.db");
    if target_db.exists() {
        return Ok(false);
    }

    let src_dir = if data.join("tayang.db").exists() {
        data.to_path_buf()
    } else if let Some(old) = legacy_data_dir(data).filter(|p| p.join("tayang.db").exists()) {
        old
    } else {
        return Ok(false);
    };

    std::fs::create_dir_all(data).map_err(|e| format!("create {}: {e}", data.display()))?;

    if src_dir != data {
        for sub in DATA_SUBDIRS {
            copy_dir_missing(&src_dir.join(sub), &data.join(sub))?;
        }
        copy_top_files(&src_dir, data)?;
    }

    let tmp_db = data.join("listune.db.migrating");
    let _ = std::fs::remove_file(&tmp_db);
    let _ = std::fs::remove_file(data.join("listune.db.migrating-wal"));
    let _ = std::fs::remove_file(data.join("listune.db.migrating-shm"));

    let src_db = src_dir.join("tayang.db");
    let result: Result<(), String> = (|| {
        let src_conn = Connection::open_with_flags(
            &src_db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("open {}: {e}", src_db.display()))?;
        let mut dst_conn = Connection::open(&tmp_db).map_err(|e| format!("open {}: {e}", tmp_db.display()))?;
        {
            let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
                .map_err(|e| format!("backup init: {e}"))?;
            backup
                .run_to_completion(100, Duration::from_millis(5), None)
                .map_err(|e| format!("backup step: {e}"))?;
        }
        rewrite_migrated_db(&dst_conn, &src_dir, data)?;
        Ok(())
    })();

    if let Err(e) = result {
        let _ = std::fs::remove_file(&tmp_db);
        return Err(e);
    }

    std::fs::rename(&tmp_db, &target_db)
        .map_err(|e| format!("rename {} -> {}: {e}", tmp_db.display(), target_db.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use tempfile::TempDir;

    #[test]
    fn resolves_sibling_legacy_data_dir_for_release_and_dev() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            legacy_data_dir(&tmp.path().join("app.listune.studio")),
            Some(tmp.path().join("app.tayang.studio"))
        );
        assert_eq!(
            legacy_data_dir(&tmp.path().join("app.listune.studio.dev")),
            Some(tmp.path().join("app.tayang.studio.dev"))
        );
        assert_eq!(legacy_data_dir(&tmp.path().join("other-folder")), None);
    }

    #[test]
    fn migrates_sibling_db_styles_jobs_and_media_folders() {
        let tmp = TempDir::new().unwrap();
        let old_dir = tmp.path().join("app.tayang.studio.dev");
        let new_dir = tmp.path().join("app.listune.studio.dev");
        std::fs::create_dir_all(old_dir.join("style-media/copied-abc")).unwrap();
        std::fs::create_dir_all(old_dir.join("style-refs/copied-abc")).unwrap();
        std::fs::create_dir_all(old_dir.join("thumbs")).unwrap();
        std::fs::create_dir_all(old_dir.join("templates")).unwrap();
        std::fs::create_dir_all(old_dir.join("claude")).unwrap();
        std::fs::write(old_dir.join("style-media/copied-abc/poster.jpg"), b"jpg").unwrap();
        std::fs::write(old_dir.join("style-refs/copied-abc/sheet-01.jpg"), b"ref").unwrap();
        std::fs::write(old_dir.join("thumbs/p1.jpg"), b"thm").unwrap();
        std::fs::write(old_dir.join("templates/a.zip"), b"zip").unwrap();
        std::fs::write(old_dir.join("claude/settings.json"), b"{}").unwrap();
        std::fs::write(old_dir.join("agent-usage.jsonl"), b"{}\n").unwrap();

        let old_pct = pct_encode_bytes(&old_dir.to_string_lossy(), true);
        let new_pct = pct_encode_bytes(&new_dir.to_string_lossy(), true);
        let old_ref = old_dir.join("style-refs/copied-abc/sheet-01.jpg");
        let new_ref = new_dir.join("style-refs/copied-abc/sheet-01.jpg");
        let old_poster = format!("http://asset.localhost/{old_pct}%5Cstyle-media%5Ccopied-abc%5Cposter.jpg");
        let new_poster = format!("http://asset.localhost/{new_pct}%5Cstyle-media%5Ccopied-abc%5Cposter.jpg");

        {
            drop(Db::open(&old_dir.join("tayang.db")).unwrap());
            let conn = Connection::open(old_dir.join("tayang.db")).unwrap();
            conn.execute(
                "INSERT INTO projects (id, title, slug, kind, format, language, folder, status, created_at, updated_at)
                 VALUES ('p1', 'My Video', 'my-video', 'explainer', '9:16', 'id', '/w/my-video', 'rendered', '2026-11-01T00:00:00Z', '2026-11-01T00:00:00Z')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO jobs (id, kind, project_id, agent_id, status, title, note, updated_at)
                 VALUES ('j1', 'build', 'p1', 'claude', 'failed', 'Run', ?1, '2026-11-01T00:00:00Z')",
                rusqlite::params![LEGACY_CLOSED_NOTE],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO styles (id, name, source, prompt, notes, created_at, preview_url, preview_video_url, refs)
                 VALUES ('copied-abc', 'Custom Style', 'copied', 'prompt', '{}', '2026-11-01T00:00:00Z', ?1, 'https://studio.listune.app/api/v1/styles/x/preview-video', ?2)",
                rusqlite::params![
                    old_poster,
                    serde_json::to_string(&vec![old_ref.to_string_lossy().into_owned()]).unwrap()
                ],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO styles (id, name, source, prompt, notes, created_at, preview_url, preview_video_url, refs)
                 VALUES ('mine-null', 'Null Refs', 'mine', 'prompt', '{}', '2026-11-01T00:00:00Z', NULL, NULL, NULL)",
                [],
            )
            .unwrap();
        }

        let ran = migrate_legacy_data(&new_dir).unwrap();
        assert!(ran);
        assert!(new_dir.join("listune.db").is_file());
        assert!(!new_dir.join("listune.db.migrating").exists());
        assert_eq!(std::fs::read(new_dir.join("style-media/copied-abc/poster.jpg")).unwrap(), b"jpg");
        assert_eq!(std::fs::read(new_dir.join("style-refs/copied-abc/sheet-01.jpg")).unwrap(), b"ref");
        assert_eq!(std::fs::read(new_dir.join("thumbs/p1.jpg")).unwrap(), b"thm");
        assert_eq!(std::fs::read(new_dir.join("templates/a.zip")).unwrap(), b"zip");
        assert_eq!(std::fs::read(new_dir.join("claude/settings.json")).unwrap(), b"{}");
        assert_eq!(std::fs::read(new_dir.join("agent-usage.jsonl")).unwrap(), b"{}\n");

        let db = Db::open(&new_dir.join("listune.db")).unwrap();
        assert_eq!(db.get_project("p1").unwrap().unwrap().title, "My Video");
        let jobs = db.list_jobs().unwrap();
        assert_eq!(jobs[0].note.as_deref(), Some(NEW_CLOSED_NOTE));
        let style = db.get_style("copied-abc").unwrap().unwrap();
        assert_eq!(style.preview_url.as_deref(), Some(new_poster.as_str()));
        // External CDN URL stays untouched!
        assert_eq!(
            style.preview_video_url.as_deref(),
            Some("https://studio.listune.app/api/v1/styles/x/preview-video")
        );
        assert_eq!(style.references, vec![new_ref.to_string_lossy().into_owned()]);

        // Idempotent: second call does nothing.
        assert!(!migrate_legacy_data(&new_dir).unwrap());
    }

    #[test]
    fn migrates_same_directory_tayang_db_to_listune_db() {
        let tmp = TempDir::new().unwrap();
        {
            drop(Db::open(&tmp.path().join("tayang.db")).unwrap());
            let conn = Connection::open(tmp.path().join("tayang.db")).unwrap();
            conn.execute(
                "INSERT INTO projects (id, title, slug, kind, format, language, folder, status, created_at, updated_at)
                 VALUES ('p2', 'Same Dir', 'same-dir', 'promo', '9:16', 'en', '/w/same', 'draft', '2026-11-01T00:00:00Z', '2026-11-01T00:00:00Z')",
                [],
            )
            .unwrap();
        }
        assert!(migrate_legacy_data(tmp.path()).unwrap());
        let db = Db::open(&tmp.path().join("listune.db")).unwrap();
        assert_eq!(db.get_project("p2").unwrap().unwrap().title, "Same Dir");
        assert!(!migrate_legacy_data(tmp.path()).unwrap());
    }

    #[test]
    fn skips_when_no_legacy_db_exists() {
        let tmp = TempDir::new().unwrap();
        let new_dir = tmp.path().join("app.listune.studio");
        assert!(!migrate_legacy_data(&new_dir).unwrap());
        assert!(!new_dir.join("listune.db").exists());
    }
}
