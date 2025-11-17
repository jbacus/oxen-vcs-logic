/// Example test file demonstrating test utilities usage
///
/// This file shows how to use the common test utilities for testing
/// the Auxin CLI wrapper. Delete or replace this file when implementing
/// actual tests as outlined in TEST_IMPLEMENTATION_PLAN.md
#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;

    #[test]
    fn example_test_fixture_creation() {
        // Create a test fixture with a Logic Pro project structure
        let fixture = TestFixture::new();

        // Verify the project structure exists
        assert!(fixture.path().exists());
        assert!(fixture.path().join("Alternatives").exists());
        assert!(fixture.path().join("Media").exists());
    }

    #[test]
    fn example_test_with_audio_file() {
        let fixture = TestFixture::new();

        // Create a 1MB audio file
        let audio_path = fixture.create_audio_file("test.wav", 1);

        // Verify file was created with correct size
        assert!(audio_path.exists());
        let metadata = std::fs::metadata(&audio_path).unwrap();
        assert_eq!(metadata.len(), 1024 * 1024);
    }

    #[test]
    fn example_test_with_metadata() {
        let fixture = TestFixture::new();

        // Create project data with metadata
        fixture.create_project_data(140, 96000, Some("Am"));

        // Verify projectData file was created
        let project_data = fixture.path().join("projectData");
        assert!(project_data.exists());

        // Verify content
        let content = std::fs::read_to_string(project_data).unwrap();
        assert!(content.contains("<tempo>140</tempo>"));
        assert!(content.contains("<sampleRate>96000</sampleRate>"));
        assert!(content.contains("<key>Am</key>"));
    }

    #[test]
    fn example_test_cleanup() {
        // TestFixture automatically cleans up when dropped
        {
            let fixture = TestFixture::new();
            let path = fixture.path().to_path_buf();
            assert!(path.exists());
        } // fixture is dropped here, temp directory is cleaned up
    }

    // Example async test (once actual oxen operations are tested)
    // #[tokio::test]
    // async fn example_async_test() {
    //     let fixture = TestFixture::new();
    //
    //     // Initialize repository
    //     // let result = oxen_ops::init(fixture.path()).await;
    //     // assert!(result.is_ok());
    // }
}
