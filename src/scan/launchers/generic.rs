use std::collections::{HashMap, HashSet};

use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;

use crate::prelude::INVALID_FILE_CHARS;

use crate::{
    resource::{
        config::Root,
        manifest::{Manifest, Store},
    },
    scan::launchers::LauncherGame,
};

fn make_fuzzy_matcher() -> fuzzy_matcher::skim::SkimMatcherV2 {
    fuzzy_matcher::skim::SkimMatcherV2::default()
        .ignore_case()
        .score_config(fuzzy_matcher::skim::SkimScoreConfig {
            penalty_case_mismatch: 0,
            ..Default::default()
        })
}

const STORE_ID: i64 = i64::MAX;
const EXACT: i64 = i64::MAX - 1;
const NORMALIZED: i64 = i64::MAX - 2;

fn compact_name(name: &str) -> String {
    name.chars()
        .filter(|&c| !c.is_whitespace() && !matches!(c, '_' | '-' | '–' | '—' | '™' | '®' | '©'))
        .flat_map(char::to_lowercase)
        .collect()
}

fn numeric_parts(name: &str) -> impl Iterator<Item = &str> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
}

fn fuzzy_match(
    matcher: &fuzzy_matcher::skim::SkimMatcherV2,
    reference: &str,
    candidate: &str,
    compact_reference: &str,
    compact_candidate: &str,
    ideal: Option<i64>,
) -> Option<i64> {
    if reference == candidate {
        return Some(EXACT);
    }
    // A close spelling match should never substitute one numbered game for another.
    if !numeric_parts(reference).eq(numeric_parts(candidate)) {
        return None;
    }
    if !compact_reference.is_empty() && compact_reference == compact_candidate {
        return Some(NORMALIZED);
    }

    // A space-consolidating regex would be better, but is too much of a performance hit.
    // Also, this is used for files/folders, so we can always ignore illegal characters.
    let candidate = candidate
        .replace(['_', '-'], " ")
        .replace(INVALID_FILE_CHARS, " ")
        .replace("    ", " ")
        .replace("   ", " ")
        .replace("  ", " ");

    let actual = matcher.fuzzy_match(reference, &candidate);
    if let (Some(ideal), Some(actual)) = (ideal, actual) {
        if actual == ideal {
            return Some(NORMALIZED);
        } else if actual > (ideal / 4 * 3) {
            return Some(actual);
        }
    }
    None
}

fn steam_install_dirs(root: &Root) -> HashMap<u32, String> {
    if root.store() != Store::Steam {
        return HashMap::new();
    }
    let Ok(path) = root.path().as_std_path_buf() else {
        return HashMap::new();
    };
    let Ok(library) = steamlocate::Library::from_dir(&path) else {
        return HashMap::new();
    };
    let install_parent = root.games_path();
    library
        .app_ids()
        .iter()
        .filter_map(|&id| {
            let app = library.app(id)?.ok()?;
            let dir = app.install_dir;
            // App manifests specify one directory under steamapps/common.
            if app.app_id != id
                || dir.trim().is_empty()
                || matches!(dir.as_str(), "." | "..")
                || dir.contains(INVALID_FILE_CHARS)
                || !install_parent.joined(&dir).is_dir()
            {
                return None;
            }
            Some((id, dir))
        })
        .collect()
}

pub fn scan(root: &Root, manifest: &Manifest, subjects: &[String]) -> HashMap<String, HashSet<LauncherGame>> {
    log::debug!("ranking installations for root: {:?}", &root);

    let install_parent = root.games_path();
    let matcher = make_fuzzy_matcher();
    let steam_dirs = steam_install_dirs(root);

    let actual_dirs: Vec<_> = install_parent
        .read_dir()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| match entry.file_type() {
                    Ok(ft) if ft.is_dir() => {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        Some((compact_name(&name), name))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    log::debug!("found {} install directories", actual_dirs.len());

    let scores: Vec<_> = subjects
        .par_iter()
        .filter_map(|name| {
            let game = manifest.0.get(name)?;
            if let Some(dir) = game.all_ids().steam(None).find_map(|id| steam_dirs.get(&id)) {
                return Some((name, dir.as_str(), STORE_ID));
            }
            let expected_dirs = game.install_dir.keys().chain(std::iter::once(name));
            let mut scores = HashMap::<&str, i64>::new();
            for expected in expected_dirs {
                if expected.contains(['/', '\\']) {
                    if install_parent.joined(expected).is_dir() {
                        return Some((name, expected.as_str(), EXACT));
                    }
                    continue;
                }

                let ideal = matcher.fuzzy_match(expected, expected);
                let compact_expected = compact_name(expected);
                for (compact_actual, actual) in &actual_dirs {
                    if actual == expected {
                        return Some((name, actual.as_str(), EXACT));
                    }
                    if let Some(score) =
                        fuzzy_match(&matcher, expected, actual, &compact_expected, compact_actual, ideal)
                    {
                        scores
                            .entry(actual)
                            .and_modify(|best| *best = (*best).max(score))
                            .or_insert(score);
                    }
                }
            }

            let (&subdir, &score) = scores.iter().max_by_key(|(_, score)| *score)?;
            if scores
                .iter()
                .any(|(&other, &other_score)| other != subdir && other_score == score)
            {
                log::debug!("[{name}] multiple install directories share the best match score");
                return None;
            }
            Some((name, subdir, score))
        })
        .collect();

    let mut by_subdir = HashMap::<&str, (i64, usize)>::new();
    for &(_, subdir, score) in &scores {
        let (best, count) = by_subdir.entry(subdir).or_insert((score, 0));
        if score > *best {
            *best = score;
            *count = 1;
        } else if score == *best {
            *count += 1;
        }
    }

    scores
        .into_iter()
        .filter_map(|(name, subdir, score)| {
            let (best, count) = by_subdir[subdir];
            if score != best || (score < EXACT && count > 1) {
                log::debug!("[{name}] ambiguous install directory: {subdir}");
                return None;
            }
            Some((
                name.clone(),
                HashSet::from([LauncherGame {
                    install_dir: Some(install_parent.joined(subdir)),
                    prefix: None,
                    platform: None,
                }]),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn steam_app_id_takes_precedence_over_the_folder_name() {
        use crate::{resource::ResourceFile, testing::repo_path};

        let root = Root::new(repo_path("tests/launchers/steam").render(), Store::Steam);
        let manifest = Manifest::load_from_string(
            r#"
            Localized Game:
              steam:
                id: 12345
            Store Folder: {}
            "#,
        )
        .unwrap();
        let detected = scan(&root, &manifest, &["Localized Game".into(), "Store Folder".into()]);

        assert_eq!(
            HashSet::from([LauncherGame {
                install_dir: Some(root.games_path().joined("Store Folder")),
                prefix: None,
                platform: None,
            }]),
            detected["Localized Game"]
        );
        assert!(!detected.contains_key("Store Folder"));
    }

    #[test]
    fn fuzzy_matching() {
        let matcher = make_fuzzy_matcher();

        for (reference, candidate, output) in vec![
            ("a", "a", Some(EXACT)),
            ("a", "b", None),
            ("Something", "Something", Some(EXACT)),
            // Too short:
            ("ab", "a", None),
            ("ab", "b", None),
            ("abc", "ab", None),
            // Long enough:
            ("abcd", "abc", Some(71)),
            ("A Fun Game", "a fun game", Some(NORMALIZED)),
            ("A Fun Game", "a  fun  game", Some(NORMALIZED)),
            ("A Fun Game", "AFunGame", Some(NORMALIZED)),
            ("A Fun Game", "A_Fun_Game", Some(NORMALIZED)),
            ("A Fun Game", "A _ Fun _ Game", Some(NORMALIZED)),
            ("A Fun Game", "a-fun-game", Some(NORMALIZED)),
            ("A Fun Game", "a - fun - game", Some(NORMALIZED)),
            ("A Fun Game", "A FUN GAME", Some(NORMALIZED)),
            ("A Fun Game!", "A Fun Game", Some(219)),
            ("A Funner Game", "A Fun Game", Some(209)),
            ("A Fun Game 2", "A Fun Game", None),
        ] {
            assert_eq!(
                output,
                fuzzy_match(
                    &matcher,
                    reference,
                    candidate,
                    &compact_name(reference),
                    &compact_name(candidate),
                    matcher.fuzzy_match(reference, reference)
                )
            );
        }
    }
}
