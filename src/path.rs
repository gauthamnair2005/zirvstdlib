//! A small, allocation-aware path type for the MOSIX virtual filesystem.
//!
//! Zirvium paths always use `/` as the separator.  There are no drive letters
//! and no Windows-style prefixes.

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::Deref;

#[repr(transparent)]
pub struct Path(str);

impl Path {
    pub fn new(value: &str) -> &Path {
        // SAFETY: `Path` is a transparent wrapper around `str`.
        unsafe { &*(value as *const str as *const Path) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|part| !part.is_empty() && *part != ".")
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.rsplit('/').find(|part| !part.is_empty())
    }

    pub fn parent(&self) -> Option<&Path> {
        let trimmed = self.0.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(0) => Some(Path::new("/")),
            Some(index) => Some(Path::new(&trimmed[..index])),
            None => None,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;
        let dot = name.rfind('.')?;
        if dot == 0 {
            None
        } else {
            Some(&name[dot + 1..])
        }
    }

    pub fn join(&self, other: &str) -> PathBuf {
        let mut buffer = PathBuf::from_str(&self.0);
        buffer.push(other);
        buffer
    }

    /// Produce a normalised absolute representation, collapsing `.` and `..`.
    pub fn normalize(&self) -> PathBuf {
        let mut parts: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
        for component in self.components() {
            match component {
                ".." => {
                    parts.pop();
                }
                "." => {}
                other => parts.push(other),
            }
        }
        let mut output = String::new();
        if self.is_absolute() {
            output.push('/');
        }
        output.push_str(&parts.join("/"));
        PathBuf(output)
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf(self.0.to_string())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Path {}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// An owned path.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathBuf(String);

impl PathBuf {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn from_str(value: &str) -> Self {
        Self(value.to_string())
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn push(&mut self, component: &str) {
        if component.starts_with('/') {
            self.0.clear();
            self.0.push_str(component);
            return;
        }
        if !self.0.is_empty() && !self.0.ends_with('/') {
            self.0.push('/');
        }
        self.0.push_str(component);
    }

    pub fn pop(&mut self) -> bool {
        let trimmed = self.0.trim_end_matches('/').to_string();
        match trimmed.rfind('/') {
            Some(0) => {
                self.0.truncate(1);
                true
            }
            Some(index) => {
                self.0.truncate(index);
                true
            }
            None => {
                let empty = self.0.is_empty();
                self.0.clear();
                !empty
            }
        }
    }

    pub fn set_extension(&mut self, extension: &str) {
        if let Some(dot) = self.0.rfind('.') {
            if let Some(slash) = self.0.rfind('/') {
                if dot < slash {
                    self.0.push('.');
                    self.0.push_str(extension);
                    return;
                }
            }
            self.0.truncate(dot);
        }
        self.0.push('.');
        self.0.push_str(extension);
    }
}

impl Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Path {
        self.as_path()
    }
}

impl From<&str> for PathBuf {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_and_parent() {
        let path = Path::new("/home/zirvium/notes.txt");
        assert!(path.is_absolute());
        assert_eq!(path.file_name(), Some("notes.txt"));
        assert_eq!(path.extension(), Some("txt"));
        assert_eq!(path.parent().unwrap().as_str(), "/home/zirvium");
        assert_eq!(path.components().collect::<alloc::vec::Vec<_>>(), alloc::vec!["home", "zirvium", "notes.txt"]);
    }

    #[test]
    fn normalize() {
        assert_eq!(Path::new("/a/./b/../c").normalize().as_str(), "/a/c");
    }

    #[test]
    fn push_pop() {
        let mut path = PathBuf::from("/home");
        path.push("user");
        path.push("file.txt");
        assert_eq!(path.as_str(), "/home/user/file.txt");
        path.set_extension("md");
        assert_eq!(path.as_str(), "/home/user/file.md");
        assert!(path.pop());
        assert_eq!(path.as_str(), "/home/user");
    }
}
