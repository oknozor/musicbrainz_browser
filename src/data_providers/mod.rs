use iced::image;
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::entity::release::{Release as MusicBrainzRelease, ReleaseSearchQuery};
use musicbrainz_rs::entity::release_group::{ReleaseGroup as MusicBrainReleaseGroup, ReleaseGroupSearchQuery};
use musicbrainz_rs::{FetchCoverart, Search};
use crate::model::{Release, ReleaseGroup};

impl ReleaseGroup {
    pub(crate) async fn search(input: String) -> Vec<MusicBrainReleaseGroup> {
        MusicBrainReleaseGroup::search(
            ReleaseGroupSearchQuery::query_builder()
                .release(&input)
                .build(),
        )
            .execute()
            .expect("Failed to query release group")
            .entities
    }

    pub(crate) async fn fetch_coverart(self) -> ReleaseGroup {
        let coverart = self
            .release
            .get_coverart()
            .front()
            .res_250()
            .execute()
            .expect("Unable to get coverart");

        match coverart {
            CoverartResponse::Url(coverart_url) => {
                let bytes = reqwest::get(&coverart_url)
                    .await
                    .expect("Unable to get coverart")
                    .bytes()
                    .await
                    .expect("Unable to get coverart bytes");
                ReleaseGroup {
                    release: self.release.clone(),
                    coverart: Some(image::Handle::from_memory(bytes.as_ref().to_vec())),
                }
            }
            _ => panic!("todo"),
        }
    }
}

impl Release {
    pub(crate) async fn search(input: String) -> Vec<MusicBrainzRelease> {
        MusicBrainzRelease::search(
            ReleaseSearchQuery::query_builder()
                .release(&input)
                .build(),
        )
            .execute()
            .expect("Failed to query release group")
            .entities
    }

    pub(crate) async fn get_coverart(self) -> Release {
        let coverart = self
            .release
            .get_coverart()
            .front()
            .res_250()
            .execute()
            .expect("Unable to get coverart");

        match coverart {
            CoverartResponse::Url(coverart_url) => {
                let bytes = reqwest::get(&coverart_url)
                    .await
                    .expect("Unable to get coverart")
                    .bytes()
                    .await
                    .expect("Unable to get coverart bytes");
                Release {
                    release: self.release.clone(),
                    coverart: Some(image::Handle::from_memory(bytes.as_ref().to_vec())),
                }
            }
            _ => panic!("todo"),
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        dbg!(error);
        Error::APIError
    }
}
