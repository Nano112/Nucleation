//! Python binding integration tests
//! 
//! Tests that validate Python bindings work correctly

use std::process::Command;
use std::path::PathBuf;

#[cfg(test)]
mod python_binding_tests {
    use super::*;

    /// Test that Python bindings can be built successfully
    #[test]
    fn test_python_build() {
        let output = Command::new("cargo")
            .args(&["build", "--features", "python"])
            .output()
            .expect("Failed to execute cargo build for Python");

        if !output.status.success() {
            panic!(
                "Python binding build failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    /// Test that generated Python bindings can be imported
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_python_import() {
        // First build the Python bindings
        test_python_build();

        // Try to import the module
        let output = Command::new("python3")
            .args(&["-c", "import nucleation; print('Import successful')"])
            .output()
            .expect("Failed to run Python import test");

        if !output.status.success() {
            panic!(
                "Python import failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        assert!(
            String::from_utf8_lossy(&output.stdout).contains("Import successful"),
            "Python import did not complete successfully"
        );
    }

    /// Test that Python tests can run
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_python_tests() {
        // Ensure we can import first
        test_python_import();

        // Run the Python test suite
        let test_script = PathBuf::from("tests/bindings/python/test_runner.py");
        let output = Command::new("python3")
            .arg(&test_script)
            .output()
            .expect("Failed to run Python tests");

        if !output.status.success() {
            panic!(
                "Python tests failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("Python tests output:\n{}", String::from_utf8_lossy(&output.stdout));
    }

    /// Test that Python stub files are generated correctly
    #[test]
    fn test_python_stub_generation() {
        // Generate all bindings (including Python stubs)
        let output = Command::new("cargo")
            .args(&["run", "--bin", "generate-bindings", "--features", "generate-bindings"])
            .output()
            .expect("Failed to generate Python bindings");

        if !output.status.success() {
            panic!(
                "Python binding generation failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Check that stub file was created
        let stub_file = PathBuf::from("python-stubs/nucleation.pyi");
        assert!(stub_file.exists(), "Python stub file was not generated at {}", stub_file.display());

        // Check stub file content
        let stub_content = std::fs::read_to_string(&stub_file)
            .expect("Failed to read stub file");

        // Verify it contains expected class definitions
        assert!(stub_content.contains("class Schematic:"), "Schematic class missing from stubs");
        assert!(stub_content.contains("class BlockState:"), "BlockState class missing from stubs");
        assert!(stub_content.contains("def load_schematic"), "load_schematic function missing from stubs");
    }

    /// Test that PyO3 module compiles correctly
    #[test]
    fn test_pyo3_compilation() {
        // This is essentially the same as test_python_build but more specific
        let output = Command::new("cargo")
            .args(&["check", "--features", "python"])
            .output()
            .expect("Failed to check Python features");

        if !output.status.success() {
            panic!(
                "PyO3 compilation check failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
