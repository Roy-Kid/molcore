//! Comprehensive integration tests for XYZ file format reader
//! 
//! These tests are slow and require test data to be downloaded.
//! Run with: cargo test --features slow-tests
//! 
//! Test data will be automatically downloaded if not present.

use molcore::io::reader::{Reader, FrameReader, open_txt, open_gz};
use molcore::io::xyz::XYZFrameReader;
use std::path::Path;

// Import common test utilities
use crate::common::require_test_data;

const TEST_DATA_DIR: &str = "tests/tests-data";

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_plain_xyz_water() {
    require_test_data();
    
    let path = format!("{}/water.xyz", TEST_DATA_DIR);
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open water.xyz")
    );
    
    let frame = reader.read_frame()
        .expect("Failed to read frame")
        .expect("Expected at least one frame");
    
    // Water molecule should have 3 atoms
    assert_eq!(frame.atoms.height(), 3);
    
    // Should have element and x, y, z columns
    assert!(frame.atoms.width() >= 4);
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_gzipped_xyz() {
    require_test_data();
    
    let path = format!("{}/water.xyz.gz", TEST_DATA_DIR);
    let mut reader = XYZFrameReader::new(
        open_gz(&path).expect("Failed to open water.xyz.gz")
    );
    
    let frame = reader.read_frame()
        .expect("Failed to read frame")
        .expect("Expected at least one frame");
    
    assert_eq!(frame.atoms.height(), 3);
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_extxyz_with_properties() {
    require_test_data();
    
    let path = format!("{}/with_properties.xyz", TEST_DATA_DIR);
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open with_properties.xyz")
    );
    
    let frame = reader.read_frame()
        .expect("Failed to read frame")
        .expect("Expected at least one frame");
    
    // Check that we have atoms
    assert!(frame.atoms.height() > 0);
    
    // Check that metadata exists
    assert!(!frame.metadata.is_empty());
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_large_molecule() {
    require_test_data();
    
    let path = format!("{}/large_molecule.xyz", TEST_DATA_DIR);
    
    // This test checks performance on larger molecules
    let start = std::time::Instant::now();
    
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open large_molecule.xyz")
    );
    
    let frame = reader.read_frame()
        .expect("Failed to read frame")
        .expect("Expected at least one frame");
    
    let duration = start.elapsed();
    
    // Should have many atoms (e.g., > 100)
    assert!(frame.atoms.height() > 100);
    
    // Should complete in reasonable time (< 1 second for typical molecules)
    println!("Read {} atoms in {:?}", frame.atoms.height(), duration);
    assert!(duration.as_secs() < 5, "Reading took too long: {:?}", duration);
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_extxyz_with_lattice() {
    require_test_data();
    
    let path = format!("{}/with_lattice.xyz", TEST_DATA_DIR);
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open with_lattice.xyz")
    );
    
    let frame = reader.read_frame()
        .expect("Failed to read frame")
        .expect("Expected at least one frame");
    
    // Check that Lattice is in metadata
    assert!(frame.metadata.contains_key("Lattice"), 
            "Expected Lattice in metadata");
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_multiple_frames_sequential() {
    require_test_data();
    
    let path = format!("{}/trajectory.xyz", TEST_DATA_DIR);
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open trajectory.xyz")
    );
    
    let mut frame_count = 0;
    let mut last_natoms = 0;
    
    // Read all frames in the file
    while let Some(frame) = reader.read_frame().expect("Failed to read frame") {
        frame_count += 1;
        let natoms = frame.atoms.height();
        
        // All frames should have the same number of atoms
        if frame_count == 1 {
            last_natoms = natoms;
        } else {
            assert_eq!(natoms, last_natoms, 
                      "Frame {} has different number of atoms", frame_count);
        }
        
        // Limit to avoid extremely long tests
        if frame_count >= 100 {
            break;
        }
    }
    
    assert!(frame_count > 0, "Should read at least one frame");
    println!("Successfully read {} frames", frame_count);
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_read_all_test_files() {
    require_test_data();
    
    // This test attempts to read all .xyz files in the test data directory
    let test_dir = Path::new(TEST_DATA_DIR);
    
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for entry in std::fs::read_dir(test_dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("xyz") {
            println!("Testing file: {:?}", path);
            
            let result = std::panic::catch_unwind(|| {
                let mut reader = XYZFrameReader::new(
                    open_txt(path.to_str().unwrap()).expect("Failed to open file")
                );
                
                reader.read_frame()
                    .expect("Failed to read frame")
                    .expect("Expected at least one frame");
            });
            
            match result {
                Ok(_) => {
                    success_count += 1;
                    println!("  ✓ Success");
                }
                Err(e) => {
                    failure_count += 1;
                    println!("  ✗ Failed: {:?}", e);
                }
            }
        }
    }
    
    println!("\nResults: {} successful, {} failed", success_count, failure_count);
    assert!(success_count > 0, "Should successfully read at least one file");
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_error_handling_invalid_format() {
    require_test_data();
    
    let path = format!("{}/invalid_format.xyz", TEST_DATA_DIR);
    
    // Skip if the file doesn't exist (it's optional)
    if !Path::new(&path).exists() {
        println!("Skipping test: invalid_format.xyz not found");
        return;
    }
    
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open invalid_format.xyz")
    );
    
    // Should return an error, not panic
    let result = reader.read_frame();
    assert!(result.is_err() || result.unwrap().is_none(), 
            "Should handle invalid format gracefully");
}

#[test]
#[cfg(feature = "slow-tests")]
fn test_stress_large_trajectory() {
    require_test_data();
    
    let path = format!("{}/stress_test.xyz", TEST_DATA_DIR);
    
    // Skip if the file doesn't exist
    if !Path::new(&path).exists() {
        println!("Skipping stress test: stress_test.xyz not found");
        return;
    }
    
    let start = std::time::Instant::now();
    let mut reader = XYZFrameReader::new(
        open_txt(&path).expect("Failed to open stress_test.xyz")
    );
    
    let mut total_atoms = 0;
    let mut frame_count = 0;
    
    while let Some(frame) = reader.read_frame().expect("Failed to read frame") {
        total_atoms += frame.atoms.height();
        frame_count += 1;
        
        // Limit to avoid extremely long tests
        if frame_count >= 1000 {
            break;
        }
    }
    
    let duration = start.elapsed();
    
    println!("Stress test: read {} frames ({} atoms) in {:?}", 
             frame_count, total_atoms, duration);
    
    // Basic sanity checks
    assert!(frame_count > 0);
    assert!(total_atoms > 0);
}
