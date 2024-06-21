use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub absolute_path: String,
    pub size: u64,
    pub is_symlink: bool,
    pub is_dir: bool,
}
pub struct FileSearchContext {
    tree: BTreeSet<FileEntry>,
}

impl FileSearchContext {
    pub fn new() -> Self {
        Self {
            tree: BTreeSet::new(),
        }
    }

    pub fn add_file(&mut self, file_entry: FileEntry) {
        self.tree.insert(file_entry);
    }

    pub fn get_files(&self) -> &BTreeSet<FileEntry> {
        &self.tree
    }
}

impl Default for FileSearchContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_type = if self.is_dir {
            "Directory"
        } else if self.is_symlink {
            "Symlink"
        } else {
            "File"
        };
        write!(f, "{}: {} ({})", file_type, self.name, self.size)
    }
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.absolute_path.cmp(&other.absolute_path)
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        (self.path == other.path) && (self.size == other.size)
    }
}

pub fn path_get_file_size(_ctx: &FileSearchContext, file: &Path) -> u64 {
    if let Ok(metadata) = fs::metadata(file) {
        metadata.len()
    } else {
        0
    }
}

pub fn path_get_file_list(ctx: &mut FileSearchContext, dir: &Path) -> io::Result<Vec<FileEntry>> {
    let mut results: Vec<FileEntry> = Vec::new();
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            if let Some(sname) = name.to_str() {
                if let Ok(apath) = fs::canonicalize(&path) {
                    let f = FileEntry {
                        name: String::from(sname),
                        path: path.to_string_lossy().into_owned(),
                        absolute_path: String::from(apath.to_string_lossy()),
                        size: path_get_file_size(ctx, &path),
                        is_symlink: path.is_symlink(),
                        is_dir: path.is_dir(),
                    };
                    ctx.add_file(f.clone());
                    results.push(f);
                }
            }
        }
    }
    Ok(results)
}

pub fn path_get_next_search_paths(
    ctx: &mut FileSearchContext,
    path: &Path,
) -> io::Result<Vec<FileEntry>> {
    let path = Path::new(&path);
    let mut results: Vec<FileEntry> = Vec::new();
    if let Ok(entries) = path_get_file_list(ctx, path) {
        for e in entries {
            if e.is_dir {
                results.push(e);
            }
        }
    }
    Ok(results)
}
