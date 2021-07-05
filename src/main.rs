use iced::{Application, Clipboard, Column, Command, Container, Element, HorizontalAlignment, image,
           Image, Length, pick_list, PickList, Row, scrollable, Scrollable, Settings, Text, text_input, TextInput};
use musicbrainz_rs::entity::{CoverartResponse};
use musicbrainz_rs::entity::artist::Artist;
use musicbrainz_rs::entity::release_group::ReleaseGroup;
use musicbrainz_rs::entity::search::SearchResult;
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
    results: Option<SearchResultState>,
}

#[derive(Debug)]
enum SearchResultState {
    ArtistResult(SearchResult<Artist>),
    ReleaseResult(ReleaseWrapper),
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Selection(SearchSelection),
    RealeaseGroupFound(Result<ReleaseWrapper, Error>),
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
        (App { state: State::default() }, Command::none())
    }

    fn title(&self) -> String {
        "Musicbrainz Browser".to_string()
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::InputChanged(value) => {
                self.state.input_value = value;
                Command::none()
            }
            Message::Search => {
                match self.state.search_kind {
                    SearchSelection::ArtistSelection => {
                        let query = Artist::query_builder()
                            .artist(&self.state.input_value)
                            .build();


                        self.state.results = Artist::search(query)
                            .execute()
                            .ok()
                            .map(|result| {
                                SearchResultState::ArtistResult(result)
                            });

                        Command::none()
                    }
                    SearchSelection::ReleaseGroupSelection => {
                        Command::perform(ReleaseWrapper::search(), Message::RealeaseGroupFound)
                    }
                }
            }
            Message::Selection(search_selection) => {
                self.state.search_kind = search_selection;
                Command::none()
            }
            Message::RealeaseGroupFound(release) => {
                if let Ok(release) = release {
                    self.state.results = Some(SearchResultState::ReleaseResult(release))
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
            .push(Container::new(pick_list)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
            )
            .push(Container::new(input)
                .width(Length::FillPortion(3))
                .height(Length::Fill)
            ).height(Length::Units(32));

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(title)
            .push(search_row);

        let results = match &self.state.results {
            Some(SearchResultState::ArtistResult(artists)) => {
                let rows: Vec<Element<Message>> = artists.entities.iter()
                    .map(|artist| Container::new(
                        Row::new()
                            .push(Text::new(format!("Artist : {}", artist.name)))
                    ).into())
                    .collect();

                Column::with_children(rows).into()
            }
            Some(SearchResultState::ReleaseResult(release)) => {
                let row = Row::new();
                let row = if let Some(coverart) = &release.coverart {
                    row.push(Image::new(coverart.clone()))
                        .push(Text::new(&release.release.title))
                } else {
                    row.push(Text::new(&release.release.title))
                };

                Column::new().push(row)
            }
            None => Column::new()
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
    async fn search() -> Result<ReleaseWrapper, Error> {
        //TODO : Search needs to be implemented in musicbrainz
        let echoes = ReleaseGroup::fetch()
            .id("ccdb3c9b-67e8-46f5-803f-026ef815ceea")
            .execute()
            .expect("Unable to get release");

        let echoes_coverart = ReleaseGroup::fetch_coverart()
            .id("ccdb3c9b-67e8-46f5-803f-026ef815ceea")
            .res_250()
            .front()
            .execute()
            .expect("Unable to get cover art");

        println!("{:?}", echoes_coverart);

        if let CoverartResponse::Url(coverart_url) = echoes_coverart {
            let bytes = reqwest::get(&coverart_url).await?.bytes().await?;
            println!("{:?}", bytes);
            Ok(ReleaseWrapper {
                release: echoes,
                coverart: Some(image::Handle::from_memory(bytes.as_ref().to_vec())),
            })
        } else {
            println!("no bytes");
            Ok(ReleaseWrapper {
                release: echoes,
                coverart: None,
            })
        }
    }
}
