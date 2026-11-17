// Integration test for SigmaOS-arch-electron
// Verifies the crate builds and links as a dependency.

use sigmaos_arch_electron::{AppConfig, ElectronManager, WindowConfig};

#[test]
fn crate_links_and_runs() {
    // Arrange
    let mut mgr = ElectronManager::new();
    let config = AppConfig::new("test-app", "/app/main.js")
        .with_window(WindowConfig::new(1, "Main"));
    mgr.register(config).unwrap();

    // Act
    let res = mgr.launch("test-app");

    // Assert
    assert!(res.is_ok());
    assert_eq!(mgr.list_apps().len(), 1);
}
