use tauri::AppHandle;
use regex::Regex;

use crate::searchers::{
    apps::AppSearcher,
    clipboard::ClipboardSearcher,
    emojis::EmojiSearcher,
    math::MathSearcher,
    shell::ShellSearcher,
    system::SystemSearcher,
    files::FileSearcher,
    web_searchers::{GoogleSearcher, YouTubeSearcher},
    SearchProvider,
};
use crate::types::{ResultItem, ResultType, SearchResult};

pub struct DefaultSearcher;

impl DefaultSearcher {
    pub fn new() -> Self {
        Self
    }
}

impl SearchProvider for DefaultSearcher {
    fn search(&self, query: &str, app: &AppHandle) -> SearchResult {
        let q = query.trim();

        // early return 
        if q.is_empty() {
            let mut results = Vec::new();
            results.extend(AppSearcher.search(q, app).results);
            return SearchResult {
                results,
                result_type: ResultType::Home,
            };
        }

        let mut combined: Vec<ResultItem> = Vec::new();

        // apps first
        combined.extend(AppSearcher.search(q, app).results);

        // files
        combined.extend(FileSearcher.search(q, app).results);

        // emojis
        if q.len() >= 1 {
            let mut res = EmojiSearcher.search(q, app).results;
            res.truncate(3);
            combined.extend(res);
        }

        // math
        let math_re = Regex::new(r"^([0-9+\-*/^().\s]+)$").unwrap();
        if math_re.is_match(q) {
            let mut res = MathSearcher.search(q, app).results;
            res.truncate(2);
            combined.extend(res);
        }

        // system
        if q.len() >= 2 {
            let mut sys = SystemSearcher.search(q, app).results;
            sys.truncate(3);
            combined.extend(sys);
        }

        // web
        if q.len() >= 2 {
            let mut g = GoogleSearcher.search(q, app).results;
            g.truncate(1);
            combined.extend(g);

            let mut yt = YouTubeSearcher.search(q, app).results;
            yt.truncate(1);
            combined.extend(yt);
        }
       
        // shell
        if q.len() >= 3 {
            let mut sh = ShellSearcher.search(q, app).results;
            sh.truncate(2);
            combined.extend(sh);
        }


        SearchResult {
            results: combined,
            result_type: ResultType::List,
        }
    }
}

