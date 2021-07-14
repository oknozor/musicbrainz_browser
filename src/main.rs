use iced::{
    image, pick_list, scrollable, text_input, Application, Clipboard, Column, Command, Container,
    Element, HorizontalAlignment, Image, Length, PickList, Row, Scrollable, Settings, Text,
    TextInput,
};
use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::entity::release_group::{ReleaseGroup, ReleaseGroupSearchQuery};
use musicbrainz_rs::entity::search::SearchResult;
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::prelude::*;

mod gui;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug)]
struct App {
    state: State,
}

#[derive(Debug, Default)]
struct State {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    pick_list: pick_list::State<SearchSelection>,
    search_kind: SearchSelection,
    search_results: Option<SearchResultState>,
}

#[derive(Debug)]
enum SearchResultState {
    ArtistResult(SearchResult<Artist>),
    ReleaseResult(Vec<ReleaseWrapper>),
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Selection(SearchSelection),
    ReleaseGroupFound(Vec<ReleaseGroup>),
    CoverArtReceived(ReleaseWrapper),
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSelection {
    ArtistSelection,
    ReleaseGroupSelection,
}

impl SearchSelection {
    const ALL: [SearchSelection; 2] = [
        SearchSelection::ArtistSelection,
        SearchSelection::ReleaseGroupSelection,
    ];
}

impl Default for SearchSelection {
    fn default() -> SearchSelection {
        SearchSelection::ArtistSelection
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

impl std::fmt::Display for SearchSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SearchSelection::ArtistSelection => "Artist",
                SearchSelection::ReleaseGroupSelection => "Release",
            }
        )
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                state: State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Musicbrainz Browser".to_string()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::InputChanged(value) => {
                self.state.input_value = value;
                Command::none()
            }
            Message::Search => match self.state.search_kind {
                SearchSelection::ArtistSelection => {
                    let query = ArtistSearchQuery::query_builder()
                        .artist(&self.state.input_value)
                        .build();

                    self.state.search_results = Artist::search(query)
                        .execute()
                        .ok()
                        .map(|result| SearchResultState::ArtistResult(result));

                    Command::none()
                }
                SearchSelection::ReleaseGroupSelection => Command::perform(
                    ReleaseWrapper::search(self.state.input_value.clone()),
                    Message::ReleaseGroupFound,
                ),
            },
            Message::Selection(search_selection) => {
                self.state.search_kind = search_selection;
                Command::none()
            }
            Message::ReleaseGroupFound(release) => {
                let release_search_result: Vec<ReleaseWrapper> = release
                    .into_iter()
                    .map(|release| ReleaseWrapper {
                        release,
                        coverart: None,
                    })
                    .collect();

                let fetch_coverart_commands =
                    release_search_result.clone().into_iter().map(|release| {
                        Command::perform(release.get_coverart(), Message::CoverArtReceived)
                    });

                self.state.search_results =
                    Some(SearchResultState::ReleaseResult(release_search_result));
                Command::batch(fetch_coverart_commands)
            }
            Message::CoverArtReceived(release_with_coverart) => {
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
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let pick_list = PickList::new(
            &mut self.state.pick_list,
            &SearchSelection::ALL[..],
            Some(self.state.search_kind),
            Message::Selection,
        );

        let input = TextInput::new(
            &mut self.state.input,
            "Search",
            &self.state.input_value,
            Message::InputChanged,
        )
        .on_submit(Message::Search);

        let title = Text::new("Musicbrainz browser")
            .width(Length::Fill)
            .size(22)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let search_row = Row::new()
            .push(
                Container::new(pick_list)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
            )
            .push(
                Container::new(input)
                    .width(Length::FillPortion(3))
                    .height(Length::Fill),
            )
            .height(Length::Units(32));

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(title)
            .push(search_row);

        let results = match &self.state.search_results {
            Some(SearchResultState::ArtistResult(artists)) => {
                let rows: Vec<Element<Message>> = artists
                    .entities
                    .iter()
                    .map(|artist| {
                        Container::new(
                            Row::new().push(Text::new(format!("Artist : {}", artist.name))),
                        )
                        .into()
                    })
                    .collect();

                Column::with_children(rows).into()
            }
            Some(SearchResultState::ReleaseResult(releases)) => {
                let rows = releases
                    .iter()
                    .map(|release| {
                        if let Some(coverart) = &release.coverart {
                            Row::new()
                                .push(Image::new(coverart.clone()))
                                .push(Text::new(&release.release.title))
                                .push(Text::new(&release.release.disambiguation))
                        } else {
                            Row::new().push(Text::new(&release.release.title))
                        }
                        .into()
                    })
                    .collect();
                Column::with_children(rows)
            }
            None => Column::new(),
        };

        Scrollable::new(&mut self.state.scroll)
            .padding(40)
            .push(content)
            .push(Container::new(results).width(Length::Fill).center_x())
            .into()
    }
}

#[derive(Debug, Clone)]
struct ReleaseWrapper {
    release: ReleaseGroup,
    coverart: Option<image::Handle>,
}

impl ReleaseWrapper {
    async fn search(input: String) -> Vec<ReleaseGroup> {
        ReleaseGroup::search(
            ReleaseGroupSearchQuery::query_builder()
                .release(&input)
                .build(),
        )
        .execute()
        .expect("Failed to query release group")
        .entities
    }

    async fn get_coverart(self) -> ReleaseWrapper {
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
                ReleaseWrapper {
                    release: self.release.clone(),
                    coverart: Some(image::Handle::from_memory(bytes.as_ref().to_vec())),
                }
            }
            _ => panic!("todo"),
        }
    }
}
