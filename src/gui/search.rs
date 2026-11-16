use fuzzy_matcher::FuzzyMatcher;
use iced::{padding, Alignment};

use crate::{
    gui::{
        button,
        common::{Message, Screen, UndoSubject},
        shortcuts::TextHistories,
        style,
        widget::{checkbox, pick_list, text, Column, Container, Element, IcedParentExt, Row},
    },
    lang::TRANSLATOR,
    resource::{config::CustomGame, manifest::Manifest},
    scan::{
        game_filter::{self, FilterKind},
        Duplication, ScanInfo,
    },
};

#[derive(Default, Clone, Eq, PartialEq)]
pub struct Filter<T> {
    active: bool,
    pub choice: T,
}

#[derive(Default)]
pub struct FilterComponent {
    pub show: bool,
    pub game_name: String,
    pub uniqueness: Filter<game_filter::Uniqueness>,
    pub completeness: Filter<game_filter::Completeness>,
    pub enablement: Filter<game_filter::Enablement>,
    pub change: Filter<game_filter::Change>,
    pub manifest: Filter<game_filter::Manifest>,
}

fn template<'a, T: 'static + Default + Copy + Eq + PartialEq + ToString>(
    filter: &'a Filter<T>,
    kind: FilterKind,
    options: &'a [T],
    message: fn(T) -> Message,
) -> Element<'a> {
    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            checkbox("", filter.active, move |enabled| Message::Filter {
                event: game_filter::Event::ToggledFilter { filter: kind, enabled },
            })
            .spacing(0),
        )
        .push(pick_list(options, Some(filter.choice), message))
        .into()
}

fn template_noncopy<T: 'static + Default + Clone + Eq + PartialEq + ToString>(
    filter: &Filter<T>,
    kind: FilterKind,
    options: Vec<T>,
    message: fn(T) -> Message,
) -> Element {
    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            checkbox("", filter.active, move |enabled| Message::Filter {
                event: game_filter::Event::ToggledFilter { filter: kind, enabled },
            })
            .spacing(0),
        )
        .push(pick_list(options, Some(filter.choice.clone()), message))
        .into()
}

fn template_with_label<T: 'static + Default + Clone + Eq + PartialEq + ToString>(
    filter: &Filter<T>,
    label: String,
    kind: FilterKind,
    options: Vec<T>,
    message: fn(T) -> Message,
) -> Element {
    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(checkbox(label, filter.active, move |enabled| Message::Filter {
            event: game_filter::Event::ToggledFilter { filter: kind, enabled },
        }))
        .push(pick_list(options, Some(filter.choice.clone()), message))
        .into()
}

impl FilterComponent {
    pub fn reset(&mut self) {
        self.game_name.clear();
        self.uniqueness.active = false;
        self.completeness.active = false;
        self.enablement.active = false;
        self.change.active = false;
        self.manifest.active = false;
    }

    pub fn is_dirty(&self) -> bool {
        !self.game_name.is_empty()
            || self.uniqueness.active
            || self.completeness.active
            || self.enablement.active
            || self.change.active
            || self.manifest.active
    }

    pub fn qualifies(
        &self,
        scan: &ScanInfo,
        manifest: &Manifest,
        enabled: bool,
        customized: bool,
        duplicated: Duplication,
        show_deselected_games: bool,
    ) -> bool {
        if !self.show {
            return true;
        }

        let fuzzy = self.game_name.is_empty()
            || fuzzy_matcher::skim::SkimMatcherV2::default()
                .fuzzy_match(&scan.game_name.to_lowercase(), &self.game_name.to_lowercase())
                .is_some();
        let unique = !self.uniqueness.active || self.uniqueness.choice.qualifies(duplicated);
        let complete = !self.completeness.active || self.completeness.choice.qualifies(scan);
        let enable = !show_deselected_games || !self.enablement.active || self.enablement.choice.qualifies(enabled);
        let changed = !self.change.active || self.change.choice.qualifies(scan);
        let manifest = !self.manifest.active
            || self
                .manifest
                .choice
                .qualifies(manifest.0.get(&scan.game_name), customized);

        fuzzy && unique && complete && changed && enable && manifest
    }

    pub fn toggle_filter(&mut self, filter: FilterKind, enabled: bool) {
        match filter {
            FilterKind::Uniqueness => self.uniqueness.active = enabled,
            FilterKind::Completeness => self.completeness.active = enabled,
            FilterKind::Enablement => self.enablement.active = enabled,
            FilterKind::Change => self.change.active = enabled,
            FilterKind::Manifest => self.manifest.active = enabled,
        }
    }

    pub fn view(
        &self,
        screen: Screen,
        histories: &TextHistories,
        show_deselected_games: bool,
        manifests: Vec<game_filter::Manifest>,
    ) -> Option<Element> {
        if !self.show {
            return None;
        }

        let content = Column::new()
            .padding(padding::left(5).right(5))
            .spacing(15)
            .push(
                Row::new()
                    .spacing(20)
                    .align_y(Alignment::Center)
                    .push(text(TRANSLATOR.filter_label()))
                    .push(histories.input(match screen {
                        Screen::Restore => UndoSubject::RestoreSearchGameName,
                        _ => UndoSubject::BackupSearchGameName,
                    }))
                    .push(button::reset_filter(self.is_dirty())),
            )
            .push(
                Row::new()
                    .spacing(15)
                    .align_y(Alignment::Center)
                    .push(template(
                        &self.uniqueness,
                        FilterKind::Uniqueness,
                        game_filter::Uniqueness::ALL,
                        move |value| Message::Filter {
                            event: game_filter::Event::EditedFilterUniqueness(value),
                        },
                    ))
                    .push(template(
                        &self.completeness,
                        FilterKind::Completeness,
                        game_filter::Completeness::ALL,
                        move |value| Message::Filter {
                            event: game_filter::Event::EditedFilterCompleteness(value),
                        },
                    ))
                    .push(template(
                        &self.change,
                        FilterKind::Change,
                        game_filter::Change::ALL,
                        move |value| Message::Filter {
                            event: game_filter::Event::EditedFilterChange(value),
                        },
                    ))
                    .push_if(show_deselected_games, || {
                        template(
                            &self.enablement,
                            FilterKind::Enablement,
                            game_filter::Enablement::ALL,
                            move |value| Message::Filter {
                                event: game_filter::Event::EditedFilterEnablement(value),
                            },
                        )
                    })
                    .push_if(manifests.len() == 2, || {
                        template_noncopy(&self.manifest, FilterKind::Manifest, manifests.clone(), move |value| {
                            Message::Filter {
                                event: game_filter::Event::EditedFilterManifest(value),
                            }
                        })
                    })
                    .push_if(manifests.len() > 2, || {
                        template_with_label(
                            &self.manifest,
                            TRANSLATOR.source_field(),
                            FilterKind::Manifest,
                            manifests,
                            move |value| Message::Filter {
                                event: game_filter::Event::EditedFilterManifest(value),
                            },
                        )
                    })
                    .wrap(),
            );

        Some(
            Container::new(
                Container::new(content)
                    .class(style::Container::GameListEntry)
                    .padding(padding::top(5).bottom(5)),
            )
            .padding(padding::left(15).right(15))
            .into(),
        )
    }
}

#[derive(Default)]
pub struct CustomGamesFilter {
    pub enabled: bool,
    pub name: String,
}

impl CustomGamesFilter {
    pub fn reset(&mut self) {
        self.name.clear();
    }

    pub fn is_dirty(&self) -> bool {
        !self.name.is_empty()
    }

    pub fn qualifies(&self, game: &CustomGame) -> bool {
        !self.enabled
            || self.name.is_empty()
            || fuzzy_matcher::skim::SkimMatcherV2::default()
                .fuzzy_match(&game.name.to_lowercase(), &self.name.to_lowercase())
                .is_some()
    }

    pub fn view<'a>(&'a self, histories: &TextHistories) -> Option<Element<'a>> {
        if !self.enabled {
            return None;
        }

        let content = Row::new()
            .padding(padding::left(5).right(5))
            .spacing(20)
            .align_y(Alignment::Center)
            .push(text(TRANSLATOR.filter_label()))
            .push(histories.input(UndoSubject::CustomGamesSearchGameName))
            .push(button::reset_filter(self.is_dirty()));

        Some(
            Container::new(
                Container::new(content)
                    .class(style::Container::GameListEntry)
                    .padding(padding::top(5).bottom(5)),
            )
            .padding(padding::left(15).right(15))
            .into(),
        )
    }
}

const INDEX_RESULT_LIMIT: usize = 6;

pub fn find_indexed_games(manifest: &Manifest, query: &str) -> Vec<String> {
    let needle = crate::scan::title::normalize_title(query);
    let mut matches = std::collections::BTreeMap::new();
    let aliases = manifest.aliases();
    if !needle.is_empty() {
        for name in manifest.0.keys() {
            let normalized = crate::scan::title::normalize_title(name);
            let rank = if normalized == needle {
                0
            } else if normalized.starts_with(&needle) {
                1
            } else if normalized.contains(&needle) {
                2
            } else {
                continue;
            };
            let canonical = aliases.get(name).unwrap_or(name);
            matches
                .entry(canonical)
                .and_modify(|best: &mut u8| *best = (*best).min(rank))
                .or_insert(rank);
        }
    }
    let mut matches: Vec<_> = matches.into_iter().collect();
    matches.sort_unstable_by(|(a, ar), (b, br)| ar.cmp(br).then_with(|| a.cmp(b)));
    matches
        .into_iter()
        .take(INDEX_RESULT_LIMIT + 1)
        .map(|(name, _)| name.clone())
        .collect()
}

pub fn indexed_games(query: &str, matches: &[String], idle: bool) -> Container<'static> {
    let mut content = Column::new().spacing(7).push(
        crate::gui::widget::TextInput::new(&TRANSLATOR.game_index_placeholder(), query)
            .id("indexed-game-search")
            .on_input(Message::SearchGameIndex)
            .padding(8),
    );

    if !query.trim().is_empty() {
        if matches.is_empty() {
            content = content.push(text(TRANSLATOR.game_index_no_matches()));
        }
        for title in matches.iter().take(INDEX_RESULT_LIMIT) {
            content = content.push(
                crate::gui::widget::Button::new(text(title.clone()))
                    .width(iced::Length::Fill)
                    .class(style::Button::NavButtonInactive)
                    .on_press_maybe(idle.then(|| Message::PreviewIndexedGame(title.clone()))),
            );
        }
        if matches.len() > INDEX_RESULT_LIMIT {
            content = content.push(text(TRANSLATOR.game_index_refine()));
        }
    }

    Container::new(content)
        .width(iced::Length::Fill)
        .max_width(580)
        .padding(10)
        .class(style::Container::Summary)
}

#[cfg(test)]
mod index_tests {
    use super::*;
    use crate::resource::ResourceFile;

    #[test]
    fn searches_before_scanning_and_merges_aliases() {
        let manifest =
            Manifest::load_from_string("Star Quest: {}\nStar Quest II: {}\nOrbit: { alias: Star Quest }\n").unwrap();

        assert_eq!(
            find_indexed_games(&manifest, "STAR QUEST"),
            ["Star Quest", "Star Quest II"]
        );
        assert_eq!(find_indexed_games(&manifest, "orbit"), ["Star Quest"]);
        assert!(find_indexed_games(&manifest, "unknown").is_empty());
    }
}
