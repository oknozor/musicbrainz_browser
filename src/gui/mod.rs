use iced::{Application, Clipboard, Column, Command, Container, Element, HorizontalAlignment, Image, Length, pick_list, PickList, Row, Scrollable, scrollable, Text, text_input, TextInput};
use musicbrainz_rs::entity::artist::{Artist};
use musicbrainz_rs::entity::search::SearchResult;
use crate::model::{Release, ReleaseGroup};
use message::Message;
use crate::gui::search::SearchSelection;

mod message;
mod search;

#[derive(Debug)]
pub struct App {
    state: State,
}

#[derive(Debug, Default)]
pub struct State {
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
    ReleaseResult(Vec<Release>),
    ReleaseGroupResult(Vec<ReleaseGroup>),
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
            Message::InputChanged(value) => self.on_input_changed(value),
            Message::Search => self.on_search_selection_changed(),
            Message::Selection(search_selection) => self.on_selection_changed(search_selection),
            Message::ReleaseGroupFound(release) => self.on_release_group_received(release),
            Message::ReleaseFound(release) => self.on_release_received(release),
            Message::ReleaseCoverArtReceived(release_with_coverart) => self.one_release_coverart_received(release_with_coverart),
            Message::ReleaseGroupCoverArtReceived(release_with_coverart) => self.on_release_group_coverart_received(release_with_coverart),
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
                                .push(Text::new(release.release.disambiguation.clone().unwrap_or("".to_string())))
                        } else {
                            Row::new().push(Text::new(&release.release.title))
                        }
                            .into()
                    })
                    .collect();
                Column::with_children(rows)
            }
            Some(SearchResultState::ReleaseGroupResult(releases)) => {
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
