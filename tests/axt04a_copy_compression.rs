use broken_divinity::copy_catalog::{CompressedCopyCatalog, CopyKey};

#[test]
fn compressed_copy_is_shorter_than_baseline() {
    let catalog = CompressedCopyCatalog;

    let baseline = catalog.baseline(CopyKey::BlockedActionHelp);
    let compressed = catalog.compressed(CopyKey::BlockedActionHelp);

    assert!(compressed.len() < baseline.len());
}

#[test]
fn compression_respects_minimum_ratio() {
    let catalog = CompressedCopyCatalog;

    let baseline = catalog.baseline(CopyKey::OverworldHint);
    let compressed = catalog.compressed(CopyKey::OverworldHint);

    let ratio = compressed.len() as f32 / baseline.len() as f32;
    assert!(ratio <= catalog.max_allowed_ratio());
}
