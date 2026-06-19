use super::service::load_runtime_status_for_bundle_dir;
use super::RuntimeBundleFolderInput;
use crate::categories;
use crate::project_categories;
use crate::storage;
use crate::storage::runtime_bundle_paths::{
    CATEGORY_CHANGE_LOG_FILE_NAME, CATEGORY_TREE_FILE_NAME, PROJECT_CATEGORY_LIST_FILE_NAME,
    SQLITE_DB_FILE_NAME,
};
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn attach_existing_runtime_bundle(
    input: RuntimeBundleFolderInput,
) -> Result<super::RuntimeStatus, String> {
    let bundle_dir = normalize_bundle_dir(&input.bundle_dir)?;
    let status = load_runtime_status_for_bundle_dir(
        &bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![format!(
            "validating existing runtime bundle at {}",
            bundle_dir.display()
        )],
    );

    if !status.is_ready {
        return Err(format_runtime_status_error(&status));
    }

    storage::init_database_at_path(&bundle_dir.join(SQLITE_DB_FILE_NAME))?;
    storage::save_user_runtime_config(&bundle_dir)?;
    Ok(load_runtime_status_for_bundle_dir(
        &bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![format!(
            "using user runtime bundle at {}",
            bundle_dir.display()
        )],
    ))
}

pub fn create_runtime_bundle_in_folder(
    input: RuntimeBundleFolderInput,
) -> Result<super::RuntimeStatus, String> {
    let bundle_dir = normalize_bundle_dir(&input.bundle_dir)?;
    create_runtime_bundle_at_dir(&bundle_dir)?;
    storage::save_user_runtime_config(&bundle_dir)?;
    Ok(load_runtime_status_for_bundle_dir(
        &bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![format!(
            "using user runtime bundle at {}",
            bundle_dir.display()
        )],
    ))
}

pub fn create_runtime_bundle_in_app_data_dir() -> Result<super::RuntimeStatus, String> {
    let bundle_dir = storage::resolve_default_runtime_bundle_dir()?;
    create_runtime_bundle_at_dir(&bundle_dir)?;
    storage::save_user_runtime_config(&bundle_dir)?;
    Ok(load_runtime_status_for_bundle_dir(
        &bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![format!(
            "using user runtime bundle at {}",
            bundle_dir.display()
        )],
    ))
}

pub fn create_runtime_bundle_backup() -> Result<super::RuntimeBackupResult, String> {
    let desktop_dir = resolve_desktop_dir()?;
    create_runtime_bundle_backup_to_dir(&desktop_dir)
}

pub(crate) fn create_runtime_bundle_backup_to_dir(
    backup_parent_dir: &Path,
) -> Result<super::RuntimeBackupResult, String> {
    let paths = storage::runtime_bundle_paths::load_resolved_runtime_paths()?;
    let status = load_runtime_status_for_bundle_dir(
        &paths.bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![],
    );

    if !status.is_ready {
        return Err(format_runtime_status_error(&status));
    }

    fs::create_dir_all(backup_parent_dir).map_err(|error| {
        format!(
            "failed to create backup target directory {}: {error}",
            backup_parent_dir.display()
        )
    })?;

    let timestamp = current_unix_timestamp()?;
    let backup_file_name = format!("System-I2-runtime-backup-{timestamp}.zip");
    let final_backup_path = backup_parent_dir.join(backup_file_name);
    let temp_backup_path = backup_parent_dir.join(format!(
        ".System-I2-runtime-backup-{timestamp}.{}.tmp",
        std::process::id()
    ));

    write_runtime_bundle_zip(&paths.bundle_dir, &temp_backup_path, &final_backup_path)?;

    fs::rename(&temp_backup_path, &final_backup_path).map_err(|error| {
        let _ = fs::remove_file(&temp_backup_path);
        format!(
            "failed to move runtime backup to {}: {error}",
            final_backup_path.display()
        )
    })?;

    Ok(super::RuntimeBackupResult {
        backup_path: final_backup_path.display().to_string(),
    })
}

fn create_runtime_bundle_at_dir(bundle_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(bundle_dir).map_err(|error| {
        format!(
            "failed to create runtime bundle directory {}: {error}",
            bundle_dir.display()
        )
    })?;

    let sqlite_db_path = bundle_dir.join(SQLITE_DB_FILE_NAME);
    let category_tree_path = bundle_dir.join(CATEGORY_TREE_FILE_NAME);
    let category_change_log_path = bundle_dir.join(CATEGORY_CHANGE_LOG_FILE_NAME);
    let project_category_list_path = bundle_dir.join(PROJECT_CATEGORY_LIST_FILE_NAME);

    reject_existing_bundle_file(&sqlite_db_path)?;
    reject_existing_bundle_file(&category_tree_path)?;
    reject_existing_bundle_file(&category_change_log_path)?;
    reject_existing_bundle_file(&project_category_list_path)?;

    storage::init_database_at_path(&sqlite_db_path)?;
    categories::write_default_task_category_tree(&category_tree_path)?;
    project_categories::write_default_project_category_list(&project_category_list_path)?;
    fs::write(&category_change_log_path, "").map_err(|error| {
        format!(
            "failed to create category change log at {}: {error}",
            category_change_log_path.display()
        )
    })?;

    let status = load_runtime_status_for_bundle_dir(
        bundle_dir,
        super::RuntimeStatusSource::UserConfig,
        vec![format!(
            "created runtime bundle at {}",
            bundle_dir.display()
        )],
    );

    if !status.is_ready {
        return Err(format_runtime_status_error(&status));
    }

    Ok(())
}

fn reject_existing_bundle_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "refusing to create runtime bundle because {} already exists",
            path.display()
        ));
    }

    Ok(())
}

fn write_runtime_bundle_zip(
    bundle_dir: &Path,
    temp_backup_path: &Path,
    final_backup_path: &Path,
) -> Result<(), String> {
    let backup_file = fs::File::create(temp_backup_path).map_err(|error| {
        format!(
            "failed to create runtime backup at {}: {error}",
            temp_backup_path.display()
        )
    })?;
    let mut zip_writer = SimpleZipWriter::new(backup_file);
    let root_name = bundle_dir
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("System-I2-runtime");

    add_dir_to_zip(
        &mut zip_writer,
        bundle_dir,
        bundle_dir,
        root_name,
        temp_backup_path,
        final_backup_path,
    )?;

    zip_writer
        .finish()
        .map_err(|error| format!("failed to finalize runtime backup zip: {error}"))?;

    Ok(())
}

fn add_dir_to_zip(
    zip_writer: &mut SimpleZipWriter<fs::File>,
    bundle_dir: &Path,
    current_dir: &Path,
    root_name: &str,
    temp_backup_path: &Path,
    final_backup_path: &Path,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir).map_err(|error| {
        format!(
            "failed to read runtime bundle directory {}: {error}",
            current_dir.display()
        )
    })?;

    for entry_result in entries {
        let entry = entry_result.map_err(|error| {
            format!(
                "failed to read runtime bundle directory entry in {}: {error}",
                current_dir.display()
            )
        })?;
        let path = entry.path();

        if path == temp_backup_path || path == final_backup_path {
            continue;
        }

        let metadata = entry.metadata().map_err(|error| {
            format!(
                "failed to read runtime bundle entry metadata at {}: {error}",
                path.display()
            )
        })?;

        if metadata.is_dir() {
            add_dir_to_zip(
                zip_writer,
                bundle_dir,
                &path,
                root_name,
                temp_backup_path,
                final_backup_path,
            )?;
            continue;
        }

        if metadata.is_file() {
            add_file_to_zip(zip_writer, bundle_dir, &path, root_name)?;
        }
    }

    Ok(())
}

fn add_file_to_zip(
    zip_writer: &mut SimpleZipWriter<fs::File>,
    bundle_dir: &Path,
    path: &Path,
    root_name: &str,
) -> Result<(), String> {
    let relative_path = path.strip_prefix(bundle_dir).map_err(|error| {
        format!(
            "failed to build relative backup path for {}: {error}",
            path.display()
        )
    })?;
    let zip_path = Path::new(root_name)
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/");
    let mut file = fs::File::open(path).map_err(|error| {
        format!(
            "failed to open runtime backup file {}: {error}",
            path.display()
        )
    })?;
    let mut contents = Vec::new();

    file.read_to_end(&mut contents).map_err(|error| {
        format!(
            "failed to read runtime backup file {}: {error}",
            path.display()
        )
    })?;

    zip_writer.start_file(&zip_path, &contents)
}

struct SimpleZipWriter<W: Write + Seek> {
    writer: W,
    entries: Vec<SimpleZipEntry>,
}

struct SimpleZipEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
}

impl<W: Write + Seek> SimpleZipWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            entries: vec![],
        }
    }

    fn start_file(&mut self, name: &str, contents: &[u8]) -> Result<(), String> {
        let name_bytes = name.as_bytes();
        let name_length = u16::try_from(name_bytes.len())
            .map_err(|_| format!("runtime backup zip entry name is too long: {name}"))?;
        let size = u32::try_from(contents.len())
            .map_err(|_| format!("runtime backup file is too large for zip32 entry: {name}"))?;
        let local_header_offset = self.current_offset()?;
        let crc32 = calculate_crc32(contents);

        write_u32(&mut self.writer, 0x0403_4b50)?;
        write_u16(&mut self.writer, 20)?;
        write_u16(&mut self.writer, 0)?;
        write_u16(&mut self.writer, 0)?;
        write_u16(&mut self.writer, 0)?;
        write_u16(&mut self.writer, zip_date_1980_01_01())?;
        write_u32(&mut self.writer, crc32)?;
        write_u32(&mut self.writer, size)?;
        write_u32(&mut self.writer, size)?;
        write_u16(&mut self.writer, name_length)?;
        write_u16(&mut self.writer, 0)?;
        self.writer
            .write_all(name_bytes)
            .map_err(|error| format!("failed to write runtime backup zip file name: {error}"))?;
        self.writer
            .write_all(contents)
            .map_err(|error| format!("failed to write runtime backup zip file data: {error}"))?;

        self.entries.push(SimpleZipEntry {
            name: name.to_string(),
            crc32,
            size,
            local_header_offset,
        });

        Ok(())
    }

    fn finish(mut self) -> Result<(), String> {
        let central_directory_offset = self.current_offset()?;
        let entry_count = u16::try_from(self.entries.len())
            .map_err(|_| "runtime backup zip has too many entries".to_string())?;

        for entry in &self.entries {
            let name_bytes = entry.name.as_bytes();
            let name_length = u16::try_from(name_bytes.len()).map_err(|_| {
                format!("runtime backup zip entry name is too long: {}", entry.name)
            })?;

            write_u32(&mut self.writer, 0x0201_4b50)?;
            write_u16(&mut self.writer, 20)?;
            write_u16(&mut self.writer, 20)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, zip_date_1980_01_01())?;
            write_u32(&mut self.writer, entry.crc32)?;
            write_u32(&mut self.writer, entry.size)?;
            write_u32(&mut self.writer, entry.size)?;
            write_u16(&mut self.writer, name_length)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, 0)?;
            write_u16(&mut self.writer, 0)?;
            write_u32(&mut self.writer, 0)?;
            write_u32(&mut self.writer, entry.local_header_offset)?;
            self.writer.write_all(name_bytes).map_err(|error| {
                format!("failed to write runtime backup central directory file name: {error}")
            })?;
        }

        let central_directory_size = self
            .current_offset()?
            .checked_sub(central_directory_offset)
            .ok_or_else(|| {
                "failed to calculate runtime backup central directory size".to_string()
            })?;

        write_u32(&mut self.writer, 0x0605_4b50)?;
        write_u16(&mut self.writer, 0)?;
        write_u16(&mut self.writer, 0)?;
        write_u16(&mut self.writer, entry_count)?;
        write_u16(&mut self.writer, entry_count)?;
        write_u32(&mut self.writer, central_directory_size)?;
        write_u32(&mut self.writer, central_directory_offset)?;
        write_u16(&mut self.writer, 0)?;
        self.writer
            .flush()
            .map_err(|error| format!("failed to flush runtime backup zip: {error}"))
    }

    fn current_offset(&mut self) -> Result<u32, String> {
        let offset = self
            .writer
            .stream_position()
            .map_err(|error| format!("failed to read runtime backup zip offset: {error}"))?;

        u32::try_from(offset)
            .map_err(|_| "runtime backup zip is too large for zip32 format".to_string())
    }
}

fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| format!("failed to write runtime backup zip u16: {error}"))
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| format!("failed to write runtime backup zip u32: {error}"))
}

fn zip_date_1980_01_01() -> u16 {
    (1 << 5) | 1
}

fn calculate_crc32(contents: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;

    for byte in contents {
        crc ^= u32::from(*byte);

        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }

    !crc
}

fn current_unix_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("failed to build runtime backup timestamp: {error}"))
}

fn resolve_desktop_dir() -> Result<PathBuf, String> {
    dirs::desktop_dir()
        .ok_or_else(|| "failed to resolve system Desktop directory for runtime backup".to_string())
}

fn normalize_bundle_dir(raw_value: &str) -> Result<PathBuf, String> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Err("runtime bundle directory path is empty".to_string());
    }

    Ok(PathBuf::from(trimmed))
}

fn format_runtime_status_error(status: &super::RuntimeStatus) -> String {
    let mut parts = Vec::<String>::new();

    if !status.missing.is_empty() {
        parts.push(format!("missing: {}", status.missing.join(", ")));
    }

    if !status.invalid.is_empty() {
        parts.push(format!("invalid: {}", status.invalid.join(" | ")));
    }

    if parts.is_empty() {
        "runtime bundle is not ready".to_string()
    } else {
        parts.join("; ")
    }
}
