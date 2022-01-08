use iced::Command;
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::entity::release_group::ReleaseGroup as MusicBrainReleaseGroup;
use musicbrainz_rs::entity::release::Release as MusicBrainzRelease;
use musicbrainz_rs::Search;
use crate::App;
use crate::gui::{SearchResultState, SearchSelection};
use crate::model::{Release, ReleaseGroup};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    Selection(SearchSelection),
    ReleaseGroupFound(Vec<MusicBrainReleaseGroup>),
    ReleaseFound(Vec<MusicBrainzRelease>),
    ReleaseCoverArtReceived(Release),
    ReleaseGroupCoverArtReceived(ReleaseGroup),
    Search,
}

impl App {
    pub(crate) fn on_release_group_received(&mut self, release: Vec<MusicBrainReleaseGroup>) -> Command<Message> {
        let release_search_result: Vec<ReleaseGroup> = release
            .into_iter()
            .map(|release| ReleaseGroup {
                release,
                coverart: None,
            })
            .collect();

        let fetch_coverart_commands =
            release_search_result.clone().into_iter().map(|release| {
                Command::perform(release.fetch_coverart(), Message::ReleaseGroupCoverArtReceived)
            });

        self.state.search_results =
            Some(SearchResultState::ReleaseGroupResult(release_search_result));
        Command::batch(fetch_coverart_commands)
    }

    pub(crate) fn on_release_received(&mut self, release: Vec<MusicBrainzRelease>) -> Command<Message> {
        let release_search_result: Vec<Release> = release
            .into_iter()
            .map(|release| Release {
                release,
                coverart: None,
            })
            .collect();

        let fetch_coverart_commands =
            release_search_result.clone().into_iter().map(|release| {
                Command::perform(release.get_coverart(), Message::ReleaseCoverArtReceived)
            });

        self.state.search_results =
            Some(SearchResultState::ReleaseResult(release_search_result));
        Command::batch(fetch_coverart_commands)
    }

    pub(crate) fn one_release_coverart_received(&mut self, release_with_coverart: Release) -> Command<Message> {
        if let Some(SearchResultState::ReleaseResult(releases)) =
        &mut self.state.search_results
        {
            let release_in_state = releases
                .iter_mut()
                .find(|r| r.release.id == release_with_coverart.release.id);
            if let Some(release_in_state) = release_in_state {
                *release_in_state = release_with_coverart;
            }
        }
        Command::none()
    }

    pub(crate) fn on_release_group_coverart_received(&mut self, release_with_coverart: ReleaseGroup) -> Command<Message> {
        if let Some(SearchResultState::ReleaseGroupResult(releases)) =
        &mut self.state.search_results
        {
            let release_in_state = releases
                .iter_mut()
                .find(|r| r.release.id == release_with_coverart.release.id);
            if let Some(release_in_state) = release_in_state {
                *release_in_state = release_with_coverart;
            }
        }
        Command::none()
    }

    pub(crate) fn on_artist_search_selected(&mut self) -> Command<Message> {
        let query = ArtistSearchQuery::query_builder()
            .artist(&self.state.input_value)
            .build();

        self.state.search_results = Artist::search(query)
            .execute()
            .ok()
            .map(|result| SearchResultState::ArtistResult(result));

        Command::none()
    }

    pub(crate) fn on_release_group_search_selected(&mut self) -> Command<Message> {
        Command::perform(
            ReleaseGroup::search(self.state.input_value.clone()),
            Message::ReleaseGroupFound,
        )
    }

    pub(crate) fn on_release_search_selected(&mut self) -> Command<Message> {
        Command::perform(
            Release::search(self.state.input_value.clone()),
            Message::ReleaseFound,
        )
    }

    pub(crate) fn on_selection_changed(&mut self, search_selection: SearchSelection) -> Command<Message> {
        self.state.search_kind = search_selection;
        Command::none()
    }

    pub(crate) fn on_search_selection_changed(&mut self) -> Command<Message> {
        match self.state.search_kind {
            SearchSelection::Artist => self.on_artist_search_selected(),
            SearchSelection::ReleaseGroup => self.on_release_group_search_selected(),
            SearchSelection::Release => self.on_release_search_selected(),
        }
    }

    pub(crate) fn on_input_changed(&mut self, value: String) -> Command<Message> {
        self.state.input_value = value;
        Command::none()
    }
}
