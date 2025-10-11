//! Common test utilities and setup
//! 
//! This module provides shared functionality for integration tests,
//! including automatic test data management.

use std::sync::Once;
use std::path::Path;
use std::process::Command;

static INIT: Once = Once::new();
static mut TEST_DATA_READY: bool = false;

const TEST_DATA_DIR: &str = "tests/tests-data";
const TEST_DATA_REPO: &str = "https://github.com/MolCrafts/tests-data.git";

/// Ensure test data is available, downloading it if necessary.
/// 
/// This function uses `std::sync::Once` to ensure the check and download
/// happens only once, even when called from multiple tests in parallel.
/// 
/// # Usage
/// 
/// Call this at the beginning of any test that needs test data:
/// 
/// ```
/// #[test]
/// fn my_test() {
///     require_test_data();
///     // Your test code here
/// }
/// ```
/// 
/// # Panics
/// 
/// Panics if:
/// - Test data directory doesn't exist and cannot be created
/// - Git is not available
/// - Download fails
pub fn require_test_data() {
    INIT.call_once(|| {
        let test_data_path = Path::new(TEST_DATA_DIR);
        
        if test_data_path.exists() {
            // Test data already exists
            println!("✓ Test data found at: {}", TEST_DATA_DIR);
            unsafe { TEST_DATA_READY = true; }
        } else {
            // Need to download test data
            println!("Test data not found. Downloading from {}...", TEST_DATA_REPO);
            
            if !is_git_available() {
                panic!("Git is not available. Please install git or manually clone the test data repository.");
            }
            
            match download_test_data() {
                Ok(_) => {
                    println!("✓ Test data downloaded successfully to: {}", TEST_DATA_DIR);
                    unsafe { TEST_DATA_READY = true; }
                }
                Err(e) => {
                    panic!("Failed to download test data: {}\n\nPlease run manually: ./scripts/fetch_testcases.sh", e);
                }
            }
        }
    });
    
    // Check if initialization was successful
    unsafe {
        if !TEST_DATA_READY {
            panic!("Test data is not available. Please check the error messages above.");
        }
    }
}

/// Check if git is available in the system
fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok()
}

/// Download test data from the repository
fn download_test_data() -> Result<(), String> {
    let tests_dir = Path::new("tests");
    
    // Ensure tests directory exists
    if !tests_dir.exists() {
        std::fs::create_dir_all(tests_dir)
            .map_err(|e| format!("Failed to create tests directory: {}", e))?;
    }
    
    // Clone the repository
    let output = Command::new("git")
        .args(&["clone", TEST_DATA_REPO, TEST_DATA_DIR])
        .output()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }
    
    // Verify the download
    let test_data_path = Path::new(TEST_DATA_DIR);
    if !test_data_path.exists() {
        return Err("Test data directory does not exist after clone".to_string());
    }
    
    Ok(())
}

/// Get the path to the test data directory
/// 
/// This does not check if the data exists. Use `require_test_data()` first.
pub fn test_data_path() -> &'static str {
    TEST_DATA_DIR
}

/// Check if test data is available without downloading
pub fn test_data_available() -> bool {
    Path::new(TEST_DATA_DIR).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_git_available() {
        // Just check if git detection works
        let available = is_git_available();
        println!("Git available: {}", available);
        // Don't assert because git might not be available in all environments
    }
    
    #[test]
    fn test_test_data_path() {
        assert_eq!(test_data_path(), "tests/tests-data");
    }
}
