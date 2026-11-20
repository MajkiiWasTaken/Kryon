use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct Workspace {
    pub root: PathBuf,
    expanded: HashSet<PathBuf>,
}

impl Workspace {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let root = path.as_ref().canonicalize()?;

        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Workspace path is not a directory",
            ));
        }

        let mut expanded = HashSet::new();
        expanded.insert(root.clone());

        Ok(Self { root, expanded })
    }

    pub fn name(&self) -> String {
        self.root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.root.display().to_string())
    }

    pub fn toggle_directory(&mut self, path: &Path) {
        if !path.is_dir() {
            return;
        }

        if !self.expanded.remove(path) {
            self.expanded.insert(path.to_path_buf());
        }
    }

    pub fn entries(&self) -> io::Result<Vec<FileEntry>> {
        let mut result = Vec::new();
        self.collect_entries(&self.root, 0, &mut result)?;

        Ok(result)
    }

    fn collect_entries(
        &self,
        directory: &Path,
        depth: usize,
        result: &mut Vec<FileEntry>,
    ) -> io::Result<()> {
        let mut children = fs::read_dir(directory)?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;

                if file_type.is_symlink() {
                    return None;
                }

                let is_dir = file_type.is_dir();

                if !is_dir && !file_type.is_file() {
                    return None;
                }

                Some((
                    entry.path(),
                    entry.file_name().to_string_lossy().into_owned(),
                    is_dir,
                ))
            })
            .collect::<Vec<_>>();

        children.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
        });

        for (path, name, is_dir) in children {
            let expanded = is_dir && self.expanded.contains(&path);

            result.push(FileEntry {
                path: path.clone(),
                name,
                is_dir,
                depth,
                expanded,
            });

            if expanded {
                if let Err(error) = self.collect_entries(&path, depth + 1, result) {
                    eprintln!("[KRYON] Cannot read {}: {}", path.display(), error);
                }
            }
        }

        Ok(())
    }
}

pub fn read_preview(path: &Path) -> io::Result<String> {
    use std::io::Read;

    const MAX_BYTES: u64 = 64 * 1024;
    const MAX_LINES: usize = 250;
    const MAX_LINE_CHARS: usize = 500;

    let metadata = fs::metadata(path)?;

    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Not a regular file",
        ));
    }

    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();

    file.take(MAX_BYTES).read_to_end(&mut bytes)?;

    if bytes.contains(&0) {
        return Ok("Binary file preview is unavailable.".into());
    }

    let content = String::from_utf8_lossy(&bytes);
    let mut preview = String::new();

    for line in content.lines().take(MAX_LINES) {
        let shortened: String = line.chars().take(MAX_LINE_CHARS).collect();

        preview.push_str(&shortened);
        preview.push('\n');
    }

    if metadata.len() > MAX_BYTES || content.lines().count() > MAX_LINES {
        preview.push_str("\n// Preview truncated. Full editor coming soon.\n");
    }

    Ok(preview)
}
