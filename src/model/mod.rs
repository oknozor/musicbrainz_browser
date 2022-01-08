use iced::image;
use musicbrainz_rs::entity::release::Release as MusicBrainzRelease;
use musicbrainz_rs::entity::release_group::ReleaseGroup as MusicBrainzReleaseGroup;

#[derive(Debug, Clone)]
pub struct Release {
    pub release: MusicBrainzRelease,
    pub coverart: Option<image::Handle>,
}

#[derive(Debug, Clone)]
pub struct ReleaseGroup {
    pub release: MusicBrainzReleaseGroup,
    pub coverart: Option<image::Handle>,
}
