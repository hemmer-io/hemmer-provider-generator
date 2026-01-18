//! Git repository cloning for SDK analysis

use crate::{AnalyzerError, Result};
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Cloned repository with automatic cleanup
pub struct ClonedRepo {
    /// Temporary directory containing the cloned repository
    temp_dir: TempDir,
    /// Path to the repository root
    repo_path: PathBuf,
}

impl ClonedRepo {
    /// Clone a git repository to a temporary directory
    ///
    /// # Arguments
    /// * `url` - Git repository URL (https:// or git@)
    /// * `branch` - Optional branch, tag, or commit to checkout
    ///
    /// # Example
    /// ```no_run
    /// use hemmer_provider_generator_analyzer::git_cloner::ClonedRepo;
    ///
    /// let repo = ClonedRepo::clone("https://github.com/awslabs/aws-sdk-rust", None).unwrap();
    /// let path = repo.path();
    /// // Analyze SDK at path...
    /// // Automatic cleanup when repo is dropped
    /// ```
    pub fn clone(url: &str, branch: Option<&str>) -> Result<Self> {
        // Create temporary directory
        let temp_dir = TempDir::new().map_err(|e| {
            AnalyzerError::Other(anyhow::anyhow!("Failed to create temp dir: {}", e))
        })?;

        // Configure fetch options
        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                eprintln!(
                    "Resolving deltas {}/{}",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                eprintln!(
                    "Receiving objects {}/{}",
                    stats.received_objects(),
                    stats.total_objects()
                );
            }
            true
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Configure repository builder
        let mut repo_builder = RepoBuilder::new();
        repo_builder.fetch_options(fetch_options);

        // Set branch if specified
        if let Some(branch_name) = branch {
            repo_builder.branch(branch_name);
        }

        // Clone repository
        eprintln!("Cloning repository: {}", url);
        if let Some(branch_name) = branch {
            eprintln!("Branch/tag: {}", branch_name);
        }

        let repo = repo_builder.clone(url, temp_dir.path()).map_err(|e| {
            AnalyzerError::Other(anyhow::anyhow!("Failed to clone repository: {}", e))
        })?;

        let repo_path = repo
            .workdir()
            .ok_or_else(|| {
                AnalyzerError::Other(anyhow::anyhow!("Repository has no working directory"))
            })?
            .to_path_buf();

        eprintln!("Successfully cloned to: {}", repo_path.display());

        Ok(Self {
            temp_dir,
            repo_path,
        })
    }

    /// Get the path to the cloned repository
    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    /// Consume the ClonedRepo and return the TempDir for manual lifetime management
    pub fn into_temp_dir(self) -> TempDir {
        use std::mem;
        use std::ptr;

        // Take ownership of temp_dir by reading it unsafely
        // This is safe because:
        // 1. We're consuming self
        // 2. We immediately forget self to prevent Drop from running
        // 3. temp_dir will never be used again after this
        let temp_dir = unsafe { ptr::read(&self.temp_dir) };
        mem::forget(self); // Prevent Drop from running
        temp_dir
    }
}

impl Drop for ClonedRepo {
    fn drop(&mut self) {
        eprintln!(
            "Cleaning up cloned repository at: {}",
            self.repo_path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires network access
    fn test_clone_repository() {
        // Clone a small public repository
        let repo = ClonedRepo::clone("https://github.com/rust-lang/rust-analyzer", Some("master"))
            .expect("Failed to clone repository");

        let path = repo.path();
        assert!(path.exists());
        assert!(path.join("Cargo.toml").exists());
    }

    #[test]
    #[ignore] // Requires network access
    fn test_clone_without_branch() {
        let repo = ClonedRepo::clone("https://github.com/rust-lang/rust-analyzer", None)
            .expect("Failed to clone repository");

        let path = repo.path();
        assert!(path.exists());
        assert!(path.join("Cargo.toml").exists());
    }

    #[test]
    fn test_invalid_url() {
        let result = ClonedRepo::clone("https://invalid-url-that-does-not-exist.com/repo", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_branch() {
        let result = ClonedRepo::clone(
            "https://github.com/rust-lang/rust-analyzer",
            Some("nonexistent-branch-12345"),
        );
        assert!(result.is_err());
    }
}
