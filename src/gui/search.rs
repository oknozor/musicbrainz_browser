#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSelection {
    Artist,
    ReleaseGroup,
    Release,
}

impl SearchSelection {
    pub(crate) const ALL: [SearchSelection; 3] = [
        SearchSelection::Artist,
        SearchSelection::ReleaseGroup,
        SearchSelection::Release,
    ];
}

impl Default for SearchSelection {
    fn default() -> SearchSelection {
        SearchSelection::Artist
    }
}

impl std::fmt::Display for SearchSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SearchSelection::Artist => "Artist",
                SearchSelection::ReleaseGroup => "Release group",
                SearchSelection::Release => "Release"
            }
        )
    }
}