use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    path: PathBuf,
    contents: String,
}

impl GeneratedFile {
    pub fn new(path: impl Into<PathBuf>, contents: impl Into<String>) -> Result<Self, EmitError> {
        let path = path.into();
        if path.is_absolute() {
            return Err(EmitError::new(
                path.display().to_string(),
                "generated file paths must be relative",
            ));
        }
        if path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return Err(EmitError::new(
                path.display().to_string(),
                "generated file paths must stay inside the output directory",
            ));
        }

        Ok(Self {
            path,
            contents: contents.into(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenFile {
    pub path: PathBuf,
    pub bytes: usize,
}

pub fn write_generated_files(
    output_dir: impl AsRef<Path>,
    files: &[GeneratedFile],
) -> Result<Vec<WrittenFile>, EmitError> {
    let output_dir = output_dir.as_ref();
    let mut sorted = files.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.path.cmp(&right.path));

    let mut written = Vec::with_capacity(sorted.len());
    for file in sorted {
        let destination = output_dir.join(&file.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                EmitError::with_source(
                    file.path.display().to_string(),
                    "failed to create generated file directory",
                    source,
                )
            })?;
        }
        fs::write(&destination, file.contents.as_bytes()).map_err(|source| {
            EmitError::with_source(
                file.path.display().to_string(),
                "failed to write generated file",
                source,
            )
        })?;
        written.push(WrittenFile {
            path: file.path.clone(),
            bytes: file.contents.len(),
        });
    }

    Ok(written)
}

#[derive(Debug)]
pub struct EmitError {
    path: String,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl EmitError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(
        path: impl Into<String>,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for EmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn std::error::Error + 'static))
    }
}
