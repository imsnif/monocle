use zellij_tile::prelude::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{self, BufRead};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

use crate::search_results::{ResultsOfSearch, SearchResult};
use crate::{SearchType, CURRENT_SEARCH_TERM, ROOT};

static MAX_FILE_SIZE_BYTES: u64 = 1000000;

#[derive(Default, Serialize, Deserialize)]
pub struct Search {
    search_type: SearchType,
    file_names: BTreeSet<String>,
    file_contents: BTreeMap<(String, usize), String>, // file_name, line_number, line
    cached_file_name_results: HashMap<String, Vec<SearchResult>>,
    cached_file_contents_results: HashMap<String, Vec<SearchResult>>,
}

impl Search {
    pub fn new(search_type: SearchType) -> Self {
        Search {
            search_type,
            ..Default::default()
        }
    }
    fn on_message(&mut self, message: String, _payload: String) {
        match serde_json::from_str::<MessageToSearch>(&message) {
            Ok(MessageToSearch::ScanFolder) => {
                self.scan_hd();
                post_message_to_plugin(PluginMessage::new_to_plugin(
                    &serde_json::to_string(&MessageToPlugin::DoneScanningFolder).unwrap(),
                    "",
                ));
            }
            Ok(MessageToSearch::Search) => {
                if let Some(current_search_term) = self.read_search_term_from_hd_cache() {
                    self.search(current_search_term);
                }
            }
            Err(e) => eprintln!("Failed to deserialize worker message {:?}", e),
        }
    }
    pub fn scan_hd(&mut self) {
        for result in Walk::new(ROOT) {
            if let Ok(entry) = result {
                self.add_file_entry(entry.path(), entry.metadata().ok());
            }
        }
    }
    pub fn search(&mut self, search_term: String) {
        let search_results_limit = 100; // artificial limit to prevent probably unwanted chaos
        let mut file_names_search_results = None;
        let mut file_contents_search_results = None;
        if let SearchType::Names | SearchType::NamesAndContents = self.search_type {
            let file_names_matches = match self.cached_file_name_results.get(&search_term) {
                Some(cached_results) => cached_results.clone(),
                None => {
                    let mut matcher = SkimMatcherV2::default().use_cache(true);
                    let results = self.search_file_names(&search_term, &mut matcher);
                    self.cached_file_name_results
                        .insert(search_term.clone(), results.clone());
                    results
                }
            };
            file_names_search_results = Some(
                ResultsOfSearch::new(search_term.clone(), file_names_matches)
                    .limit_search_results(search_results_limit),
            );
        };
        if let SearchType::Contents | SearchType::NamesAndContents = self.search_type {
            let file_contents_matches = match self.cached_file_contents_results.get(&search_term) {
                Some(cached_results) => cached_results.clone(),
                None => {
                    let mut matcher = SkimMatcherV2::default().use_cache(true);
                    let results = self.search_file_contents(&search_term, &mut matcher);
                    self.cached_file_contents_results
                        .insert(search_term.clone(), results.clone());
                    results
                }
            };
            file_contents_search_results = Some(
                ResultsOfSearch::new(search_term.clone(), file_contents_matches)
                    .limit_search_results(search_results_limit),
            );
        };

        // if the search term changed before we finished, let's search again!
        if let Some(current_search_term) = self.read_search_term_from_hd_cache() {
            if current_search_term != search_term {
                return self.search(current_search_term.into());
            }
        }
        if let Some(file_names_search_results) = file_names_search_results {
            post_message_to_plugin(PluginMessage::new_to_plugin(
                &serde_json::to_string(&MessageToPlugin::UpdateFileNameSearchResults).unwrap(),
                &serde_json::to_string(&file_names_search_results).unwrap(),
            ));
        }
        if let Some(file_contents_search_results) = file_contents_search_results {
            post_message_to_plugin(PluginMessage::new_to_plugin(
                &serde_json::to_string(&MessageToPlugin::UpdateFileContentsSearchResults).unwrap(),
                &serde_json::to_string(&file_contents_search_results).unwrap(),
            ));
        }
    }
    fn add_file_entry(&mut self, file_name: &Path, file_metadata: Option<std::fs::Metadata>) {
        let file_path = file_name.display().to_string();
        let file_path_stripped_prefix = self.strip_file_prefix(&file_name);

        self.file_names.insert(file_path_stripped_prefix.clone());
        if let SearchType::NamesAndContents | SearchType::Contents = self.search_type {
            if file_metadata.map(|f| f.is_file()).unwrap_or(false) {
                if let Ok(file) = std::fs::File::open(&file_path) {
                    let file_size = file
                        .metadata()
                        .map(|f| f.len())
                        .unwrap_or(MAX_FILE_SIZE_BYTES);
                    if file_size >= MAX_FILE_SIZE_BYTES {
                        eprintln!(
                            "File {} too large, not indexing its contents",
                            file_name.display()
                        );
                        return;
                    }
                    let lines = io::BufReader::with_capacity(file_size as usize, file).lines();
                    for (index, line) in lines.enumerate() {
                        match line {
                            Ok(line) => {
                                self.file_contents.insert(
                                    (file_path_stripped_prefix.clone(), index + 1),
                                    String::from_utf8_lossy(&strip_ansi_escapes::strip(line))
                                        .to_string(),
                                );
                            }
                            Err(_) => {
                                break; // probably a binary file, skip it
                            }
                        }
                    }
                }
            }
        }
    }
    fn search_file_names(
        &self,
        search_term: &str,
        matcher: &mut SkimMatcherV2,
    ) -> Vec<SearchResult> {
        let mut matches = vec![];
        for entry in &self.file_names {
            if let Some((score, indices)) = matcher.fuzzy_indices(&entry, &search_term) {
                matches.push(SearchResult::new_file_name(
                    score,
                    indices,
                    entry.to_owned(),
                ));
            }
        }
        matches
    }
    fn search_file_contents(
        &self,
        search_term: &str,
        matcher: &mut SkimMatcherV2,
    ) -> Vec<SearchResult> {
        let mut matches = vec![];
        for ((file_name, line_number), line_entry) in &self.file_contents {
            if let Some((score, indices)) = matcher.fuzzy_indices(&line_entry, &search_term) {
                matches.push(SearchResult::new_file_line(
                    score,
                    indices,
                    file_name.clone(),
                    line_entry.clone(),
                    *line_number,
                ));
            }
        }
        matches
    }
    fn strip_file_prefix(&self, file_name: &Path) -> String {
        let mut file_path_stripped_prefix = file_name.display().to_string().split_off(ROOT.width());
        if file_path_stripped_prefix.starts_with('/') {
            file_path_stripped_prefix.remove(0);
        }
        file_path_stripped_prefix
    }
    fn read_search_term_from_hd_cache(&self) -> Option<String> {
        match std::fs::read(CURRENT_SEARCH_TERM) {
            Ok(current_search_term) => {
                Some(String::from_utf8_lossy(&current_search_term).to_string())
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum MessageToSearch {
    ScanFolder,
    Search,
}

#[derive(Serialize, Deserialize)]
pub enum MessageToPlugin {
    UpdateFileNameSearchResults,
    UpdateFileContentsSearchResults,
    DoneScanningFolder,
}

#[derive(Serialize, Deserialize)]
pub struct FileNameWorker {
    search: Search,
}

impl Default for FileNameWorker {
    fn default() -> Self {
        FileNameWorker {
            search: Search::new(SearchType::Names),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileContentsWorker {
    search: Search,
}

impl Default for FileContentsWorker {
    fn default() -> Self {
        FileContentsWorker {
            search: Search::new(SearchType::Contents),
        }
    }
}

impl<'de> ZellijWorker<'de> for FileNameWorker {
    fn on_message(&mut self, message: String, payload: String) {
        self.search.on_message(message, payload);
    }
}

impl<'de> ZellijWorker<'de> for FileContentsWorker {
    fn on_message(&mut self, message: String, payload: String) {
        self.search.on_message(message, payload);
    }
}
