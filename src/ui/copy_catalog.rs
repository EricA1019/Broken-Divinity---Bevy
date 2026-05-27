const MAX_ALLOWED_COMPRESSION_RATIO: f32 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKey {
    BlockedActionHelp,
    OverworldHint,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompressedCopyCatalog;

impl CompressedCopyCatalog {
    pub fn baseline(&self, key: CopyKey) -> &'static str {
        match key {
            CopyKey::BlockedActionHelp => {
                "Action failed because the target is invalid. Check distance and path before trying again."
            }
            CopyKey::OverworldHint => {
                "Open the overworld map, pick a destination, then confirm travel to continue progression."
            }
        }
    }

    pub fn compressed(&self, key: CopyKey) -> &'static str {
        match key {
            CopyKey::BlockedActionHelp => "Action failed. Check distance and path, then retry.",
            CopyKey::OverworldHint => "Open map, choose destination, confirm travel.",
        }
    }

    pub fn max_allowed_ratio(&self) -> f32 {
        MAX_ALLOWED_COMPRESSION_RATIO
    }
}
