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

                // Symbolické odkazy v první verzi nepřecházíme.
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
                self.collect_entries(&path, depth + 1, result)?;
            }
        }

        Ok(())
    }
}

pub fn read_preview(path: &Path) -> io::Result<String> {
    const MAX_PREVIEW_BYTES: u64 = 256 * 1024;

    let metadata = fs::metadata(path)?;

    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Not a regular file",
        ));
    }

    if metadata.len() > MAX_PREVIEW_BYTES {
        return Ok("File is too large for the current preview.".into());
    }

    let bytes = fs::read(path)?;

    if bytes.contains(&0) {
        return Ok("Binary file preview is unavailable.".into());
    }

    String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "File is not valid UTF-8"))
}
