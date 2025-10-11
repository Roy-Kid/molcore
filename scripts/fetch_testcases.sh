#!/usr/bin/env bash
# Fetch test data from MolCrafts tests-data repository
# This script clones or updates the test data repository

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TESTS_DIR="$PROJECT_ROOT/tests"
TEST_DATA_REPO="https://github.com/MolCrafts/tests-data.git"
TEST_DATA_DIR="$TESTS_DIR/tests-data"

echo "=== MolCore Test Data Fetch Script ==="
echo "Project root: $PROJECT_ROOT"
echo "Tests directory: $TESTS_DIR"
echo "Test data will be stored in: $TEST_DATA_DIR"
echo

# Create tests directory if it doesn't exist
mkdir -p "$TESTS_DIR"

# Check if the test data directory already exists
if [ -d "$TEST_DATA_DIR" ]; then
    echo "Test data directory already exists. Updating..."
    cd "$TEST_DATA_DIR"
    
    # Check if it's a git repository
    if [ -d ".git" ]; then
        echo "Pulling latest changes from repository..."
        git pull origin main || git pull origin master || {
            echo "Warning: Could not pull changes. Continuing with existing data."
        }
        echo "✓ Test data updated successfully"
    else
        echo "Warning: $TEST_DATA_DIR exists but is not a git repository."
        echo "Please remove it manually if you want to re-clone."
    fi
else
    echo "Cloning test data repository..."
    cd "$TESTS_DIR"
    
    if git clone "$TEST_DATA_REPO" tests-data; then
        echo "✓ Test data cloned successfully"
    else
        echo "✗ Failed to clone test data repository"
        echo "Please check your internet connection and try again."
        exit 1
    fi
fi

echo
echo "=== Test Data Fetch Complete ==="
echo

# Display some information about the downloaded data
if [ -d "$TEST_DATA_DIR" ]; then
    echo "Test data location: $TEST_DATA_DIR"
    echo
    echo "Contents:"
    ls -lh "$TEST_DATA_DIR" 2>/dev/null || echo "  (empty or no files)"
    echo
    
    # Count XYZ files if any
    xyz_count=$(find "$TEST_DATA_DIR" -name "*.xyz" -o -name "*.xyz.gz" 2>/dev/null | wc -l)
    if [ "$xyz_count" -gt 0 ]; then
        echo "Found $xyz_count XYZ test files"
    fi
    
    echo
    echo "You can now run the slow tests with:"
    echo "  cargo test --features slow-tests"
    echo
fi
