use tokio::io::sink;

use super::NodejsAccess;

#[tokio::test]
async fn it_fetches_all_versions() {
    let versions = NodejsAccess::default().get_versions().await.unwrap();
    assert!(!versions.is_empty());
}

#[tokio::test]
async fn it_downloads_a_specific_version() {
    let mut writer = sink();
    let bytes_written = NodejsAccess::default()
        .download_version("15.0.0", &mut writer)
        .await
        .unwrap();
    assert!(bytes_written > 0);
}
