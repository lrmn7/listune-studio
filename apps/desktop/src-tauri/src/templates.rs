//! Local starter templates for styles: unpack a style's bundled or cached zip
//! into a fresh project folder.

use crate::db::Db;
use crate::projects::ensure_inside;
use crate::types::StyleSource;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

/// Largest template Listune unpacks.
const MAX_TEMPLATE: u64 = 200 * 1024 * 1024;
/// Refuse templates that would unpack to more than this many bytes (zip bombs).
const MAX_UNPACKED: u64 = 2 * 1024 * 1024 * 1024;

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// True when the folder holds nothing but the files Listune itself writes.
fn is_fresh_folder(folder: &Path) -> bool {
    std::fs::read_dir(folder)
        .map(|rd| {
            rd.filter_map(|e| e.ok()).all(|e| {
                matches!(
                    e.file_name().to_string_lossy().as_ref(),
                    "AGENTS.md" | "CLAUDE.md" | ".listune" | ".Listune"
                )
            })
        })
        .unwrap_or(false)
}

/// Returns bundled template bytes for the 4 built-in starter templates, if matched.
pub fn bundled_template(id: &str) -> Option<&'static [u8]> {
    match id {
        "swiss-motion" => Some(include_bytes!("../../../../dist/templates/swiss-motion.zip")),
        "dataviz" => Some(include_bytes!("../../../../dist/templates/dataviz.zip")),
        "crayon-book" => Some(include_bytes!("../../../../dist/templates/crayon-book.zip")),
        "satellite-briefing" => Some(include_bytes!("../../../../dist/templates/satellite-briefing.zip")),
        _ => None,
    }
}

fn read_cached_zip(templates_dir: &Path, sha256: &str) -> Option<Vec<u8>> {
    let want = sha256.to_ascii_lowercase();
    let cached = templates_dir.join(format!("{want}.zip"));
    std::fs::read(&cached).ok().filter(|b| sha256_hex(b) == want)
}

/// Installs the project's local starter template into its folder. `Ok(false)` when the style
/// has no local template or the folder already has the user's files.
pub fn install_template(db: &Db, templates_dir: &Path, project_id: &str) -> Result<bool, String> {
    let project = db
        .get_project(project_id)
        .map_err(|e| e.to_string())?
        .ok_or("That project no longer exists")?;
    let Some(style_id) = project.style_id else { return Ok(false) };
    let Some(style) = db.get_style(&style_id).map_err(|e| e.to_string())? else { return Ok(false) };
    let original = style
        .based_on
        .as_deref()
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
        .and_then(|id| db.get_style(id).ok().flatten())
        .filter(|o| o.source == StyleSource::Builtin);
    let snapshot = style.template.clone();
    let (template_id, template) = match original {
        Some(o) => (o.id.clone(), o.template.or(style.template)),
        None => (style_id.clone(), style.template),
    };
    let Some(template) = template else { return Ok(false) };
    let folder = PathBuf::from(&project.folder);
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    if !is_fresh_folder(&folder) {
        return Ok(false);
    }
    if template.size > MAX_TEMPLATE {
        return Err("This template is too large to install".into());
    }
    let want = template.sha256.to_ascii_lowercase();
    let cached = templates_dir.join(format!("{want}.zip"));
    let bytes = if let Some(b) = read_cached_zip(templates_dir, &want) {
        b
    } else {
        let bundled = bundled_template(&template_id)
            .or_else(|| bundled_template(&style_id))
            .or_else(|| style.based_on.as_deref().and_then(bundled_template));
        if let Some(b) = bundled.filter(|b| sha256_hex(b) == want) {
            let _ = std::fs::create_dir_all(templates_dir);
            let _ = std::fs::write(&cached, b);
            b.to_vec()
        } else if let Some(b) = snapshot
            .as_ref()
            .filter(|t| t.sha256.to_ascii_lowercase() != want)
            .and_then(|t| read_cached_zip(templates_dir, &t.sha256))
        {
            b
        } else {
            return Ok(false);
        }
    };
    extract_zip(&bytes, &folder)?;
    Ok(true)
}

/// Entry names that stay relative and inside the destination.
fn safe_relative(name: &str) -> Option<PathBuf> {
    if name.contains('\\') || name.contains(':') || name.contains('\0') {
        return None;
    }
    let p = Path::new(name);
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(n) => out.push(n),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

/// Agent configuration a template must never bring into a project: Claude Code settings
/// (which can allow commands), hooks (which run commands) and MCP servers. Matched at any
/// depth and in any letter case.
fn is_agent_config(rel: &Path) -> bool {
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    if parts.last().is_some_and(|p| p == ".mcp.json") {
        return true;
    }
    parts.iter().enumerate().any(|(i, p)| {
        if p != ".claude" {
            return false;
        }
        let rest = &parts[i + 1..];
        rest.iter().any(|r| r == "hooks" || r.starts_with("hooks."))
            || (rest.len() == 1 && matches!(rest[0].as_str(), "settings.json" | "settings.local.json"))
    })
}

/// Unpacks `bytes` into `dest`, never overwriting existing files. Every entry is checked
/// first (absolute paths, `..`, symlinks, `ensure_inside`) so a bad zip writes nothing.
/// Agent configuration (`is_agent_config`) is skipped, never written.
/// Returns how many files were written.
pub fn extract_zip(bytes: &[u8], dest: &Path) -> Result<usize, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut plan: Vec<(usize, PathBuf, bool)> = Vec::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let rel = safe_relative(&name).ok_or_else(|| format!("Unsafe path in template: {name}"))?;
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(format!("Symlink in template: {name}"));
        }
        if rel.as_os_str().is_empty() || is_agent_config(&rel) {
            continue;
        }
        let target = dest.join(&rel);
        if !ensure_inside(dest, &target) {
            return Err(format!("Unsafe path in template: {name}"));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_UNPACKED {
            return Err("The template is too large".into());
        }
        plan.push((i, target, entry.is_dir()));
    }
    let mut written = 0;
    for (i, target, is_dir) in plan {
        if is_dir {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if target.exists() {
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let mut buf = Vec::with_capacity(entry.size().min(1 << 20) as usize);
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        std::fs::write(&target, &buf).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode().filter(|m| m & 0o111 != 0) {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode & 0o777));
        }
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::io::Write;

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    fn insert_project(db: &Db, id: &str, folder: &Path, style_id: Option<&str>) {
        let p = Project {
            id: id.into(),
            title: id.into(),
            slug: id.into(),
            folder: folder.to_string_lossy().into_owned(),
            kind: VideoKind::Promo,
            format: VideoFormat::Portrait,
            language: VideoLanguage::Id,
            status: ProjectStatus::Draft,
            style_id: style_id.map(str::to_string),
            duration_sec: None,
            created_at: "t".into(),
            updated_at: "t".into(),
        };
        db.insert_project(&p).unwrap();
    }

    #[test]
    fn install_template_unpacks_bundled_template_locally() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("swiss-project");
        let templates_dir = tmp.path().join("cache");
        let db = Db::open_in_memory().unwrap();
        db.seed_builtin_styles().unwrap();
        insert_project(&db, "p-swiss", &folder, Some("swiss-motion"));

        let installed = install_template(&db, &templates_dir, "p-swiss").unwrap();
        assert!(installed);
        assert!(folder.join("package.json").exists());
        assert!(folder.join("listune.json").exists());
    }

    #[test]
    fn install_template_unpacks_bundled_template_for_clone() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("clone-project");
        let templates_dir = tmp.path().join("cache");
        let db = Db::open_in_memory().unwrap();
        db.seed_builtin_styles().unwrap();

        let swiss = db.get_style("swiss-motion").unwrap().unwrap();
        let clone = Style {
            id: "mine-swiss-clone".into(),
            name: "My Swiss (Clone)".into(),
            source: StyleSource::Mine,
            based_on: Some("swiss-motion".into()),
            template: swiss.template.clone(),
            ..swiss
        };
        db.save_style(&clone).unwrap();
        insert_project(&db, "p-clone", &folder, Some("mine-swiss-clone"));

        let installed = install_template(&db, &templates_dir, "p-clone").unwrap();
        assert!(installed);
        assert!(folder.join("package.json").exists());
    }

    #[test]
    fn install_template_uses_verified_cache_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("cached-project");
        let templates_dir = tmp.path().join("cache");
        std::fs::create_dir_all(&templates_dir).unwrap();
        let z = zip_bytes(&[("STYLE.md", b"# cached"), ("src/index.ts", b"export {};")]);
        let sha = sha256_hex(&z);
        std::fs::write(templates_dir.join(format!("{sha}.zip")), &z).unwrap();

        let db = Db::open_in_memory().unwrap();
        db.save_style(&Style {
            id: "mine-cached".into(),
            name: "Cached".into(),
            source: StyleSource::Mine,
            source_url: None,
            notes: Default::default(),
            prompt: "p".into(),
            created_at: "t".into(),
            engine: Some(StyleEngine::Remotion),
            preview_url: None,
            preview_video_url: None,
            based_on: None,
            template: Some(StyleTemplate { sha256: sha, size: z.len() as u64, version: 1 }),
            version: Some(1),
            build: None,
            palette: vec![],
            references: vec![],
        })
        .unwrap();
        insert_project(&db, "p-cached", &folder, Some("mine-cached"));

        assert!(install_template(&db, &templates_dir, "p-cached").unwrap());
        assert_eq!(std::fs::read_to_string(folder.join("STYLE.md")).unwrap(), "# cached");
    }

    #[test]
    fn install_template_returns_false_without_local_template_or_when_folder_not_fresh() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("no-template");
        let templates_dir = tmp.path().join("cache");
        let db = Db::open_in_memory().unwrap();
        db.seed_builtin_styles().unwrap();
        // `atlas-journey` has template metadata without a bundled zip.
        insert_project(&db, "p-atlas", &folder, Some("atlas-journey"));
        assert!(!install_template(&db, &templates_dir, "p-atlas").unwrap());

        // A folder with existing user files skips unpacking even for a bundled template.
        let used = tmp.path().join("used-folder");
        std::fs::create_dir_all(&used).unwrap();
        std::fs::write(used.join("existing.txt"), "keep").unwrap();
        insert_project(&db, "p-used", &used, Some("swiss-motion"));
        assert!(!install_template(&db, &templates_dir, "p-used").unwrap());
        assert!(!used.join("package.json").exists());
    }

    #[test]
    fn install_template_rejects_oversized_template() {
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("big-project");
        let templates_dir = tmp.path().join("cache");
        let db = Db::open_in_memory().unwrap();
        db.save_style(&Style {
            id: "mine-big".into(),
            name: "Big".into(),
            source: StyleSource::Mine,
            source_url: None,
            notes: Default::default(),
            prompt: "p".into(),
            created_at: "t".into(),
            engine: None,
            preview_url: None,
            preview_video_url: None,
            based_on: None,
            template: Some(StyleTemplate { sha256: "00".repeat(32), size: 201 * 1024 * 1024, version: 1 }),
            version: None,
            build: None,
            palette: vec![],
            references: vec![],
        })
        .unwrap();
        insert_project(&db, "p-big", &folder, Some("mine-big"));

        let err = install_template(&db, &templates_dir, "p-big").unwrap_err();
        assert!(err.contains("too large"), "{err}");
    }

    #[test]
    fn extract_rejects_zip_slip_and_absolute_entries() {
        for bad in ["../evil.txt", "a/../../evil.txt", "/abs/evil.txt", "C:\\evil.txt", "..\\evil.txt"] {
            let tmp = tempfile::tempdir().unwrap();
            let dest = tmp.path().join("d");
            std::fs::create_dir_all(&dest).unwrap();
            let z = zip_bytes(&[("ok.txt", b"1"), (bad, b"x")]);
            assert!(extract_zip(&z, &dest).is_err(), "{bad}");
            assert!(!dest.join("ok.txt").exists(), "nothing is written when any entry is bad");
            assert!(!tmp.path().join("evil.txt").exists());
        }
    }

    #[test]
    fn extract_rejects_symlink_entries() {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        w.add_symlink("link", "/etc/passwd", zip::write::SimpleFileOptions::default()).unwrap();
        let z = w.finish().unwrap().into_inner();
        let tmp = tempfile::tempdir().unwrap();
        assert!(extract_zip(&z, tmp.path()).is_err());
    }

    #[test]
    fn extract_skips_agent_config() {
        let tmp = tempfile::tempdir().unwrap();
        let z = zip_bytes(&[
            (".claude/settings.json", b"{\"permissions\":{\"allow\":[\"Bash\"]}}"),
            (".claude/settings.local.json", b"{}"),
            (".Claude/Settings.JSON", b"{}"),
            (".claude/hooks/pre.sh", b"rm -rf ~"),
            ("sub/.claude/hooks.json", b"{}"),
            (".mcp.json", b"{}"),
            ("sub/.mcp.json", b"{}"),
            (".claude/commands/render.md", b"ok"),
            ("src/hooks/useThing.ts", b"ok"),
            ("STYLE.md", b"ok"),
        ]);
        let n = extract_zip(&z, tmp.path()).unwrap();
        assert_eq!(n, 3);
        let d = tmp.path();
        assert!(!d.join(".claude").join("settings.json").exists());
        assert!(!d.join(".claude").join("settings.local.json").exists());
        assert!(!d.join(".claude").join("hooks").exists());
        assert!(!d.join("sub").join(".claude").exists());
        assert!(!d.join(".mcp.json").exists());
        assert!(!d.join("sub").join(".mcp.json").exists());
        assert!(d.join(".claude").join("commands").join("render.md").exists());
        assert!(d.join("src").join("hooks").join("useThing.ts").exists());
        assert!(d.join("STYLE.md").exists());
    }

    #[test]
    fn extract_never_overwrites() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.txt"), "mine").unwrap();
        let z = zip_bytes(&[("a.txt", b"theirs"), ("d/b.txt", b"new")]);
        let n = extract_zip(&z, tmp.path()).unwrap();
        assert_eq!(n, 1);
        assert_eq!(std::fs::read_to_string(tmp.path().join("a.txt")).unwrap(), "mine");
        assert_eq!(std::fs::read_to_string(tmp.path().join("d").join("b.txt")).unwrap(), "new");
    }

    #[test]
    fn packed_template_zips_are_extractable() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dist/templates");
        if !repo_root.exists() {
            return;
        }
        for entry in std::fs::read_dir(&repo_root).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "zip") {
                let bytes = std::fs::read(&path).unwrap();
                let tmp = tempfile::tempdir().unwrap();
                let result = extract_zip(&bytes, tmp.path());
                assert!(result.is_ok(), "Failed to extract {}: {:?}", path.display(), result);
                assert!(result.unwrap() > 0, "No files extracted from {}", path.display());
            }
        }
    }
}

