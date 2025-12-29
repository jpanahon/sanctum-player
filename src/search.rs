use crate::Song;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct Search {
    pub modal: bool,
    pub matcher: SkimMatcherV2,
    pub results: Vec<(usize, i64)>,
    pub query: String,
}

impl Search {
    pub fn search_query(&mut self, songs: &Vec<Song>) -> Vec<(usize, i64)> {
        let query = self.query.trim();

        if query.is_empty() || query.len() < 2 {
            return Vec::new();
        }

        let mut results = Vec::new();

        for (index, song) in songs.iter().enumerate() {
            if let Some(score) = self.matcher.fuzzy_match(&song.search_key, query) {
                results.push((index, score));
            }
        }

        results.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((_, best)) = results.first() {
            let cutoff = best / 2;
            results.retain(|(_, score)| *score >= cutoff);
        }

        results
    }

    pub fn handle_query(&mut self, songs: &Vec<Song>) {
        self.results = self.search_query(songs);
    }

    pub fn open_modal(&mut self) {
        if !self.results.is_empty() {
            self.query.clear();
            self.results.clear();
        }

        self.modal = true;
    }

    pub fn close_modal(&mut self) {
        self.modal = false;
    }
}

impl Default for Search {
    fn default() -> Self {
        Self {
            modal: false,
            matcher: SkimMatcherV2::default().ignore_case(),
            results: Vec::new(),
            query: String::new(),
        }
    }
}
