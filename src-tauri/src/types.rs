use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Action {
    pub id: String,
    pub name: String,
    #[serde(skip)]
    pub data: ActionData,
}

impl Action {
    pub fn new(name: impl Into<String>, data: ActionData) -> Self {
        Self {
            id: String::new(),
            name: name.into(),
            data,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ResultItem {
    pub name: String,
    pub actions: Vec<Action>,
    pub description: Option<String>,
    pub icon: Option<String>,
    /// For image clipboard entries: base64 PNG thumbnail for the UI to render
    pub thumbnail: Option<String>,
    /// OCR text extracted from clipboard images, for display in the preview panel
    pub ocr_text: Option<String>,
    pub pinned: bool,
    pub group: Option<String>,
    pub draggable_path: Option<String>,
}

impl ResultItem {
    pub fn new(name: impl Into<String>, actions: Vec<Action>) -> Self {
        Self {
            name: name.into(),
            actions,
            description: None,
            icon: None,
            thumbnail: None,
            ocr_text: None,
            pinned: false,
            group: None,
            draggable_path: None,
        }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(i.into());
        self
    }

    pub fn thumbnail(mut self, t: impl Into<String>) -> Self {
        self.thumbnail = Some(t.into());
        self
    }

    pub fn ocr_text(mut self, t: impl Into<String>) -> Self {
        self.ocr_text = Some(t.into());
        self
    }

    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }

    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    pub fn draggable_path(mut self, p: impl Into<String>) -> Self {
        self.draggable_path = Some(p.into());
        self
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum ResultType {
    List,
    Grid,
    WebSearch,
    Markdown,
    Clipboard,
    ColorPicker,
    Home,
    Media,
    Math,
    Camera,
    Ai,
    Screenshots,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub results: Vec<ResultItem>,
    pub result_type: ResultType,
    #[serde(default)]
    pub searcher: String,
}

impl Default for SearchResult {
    fn default() -> Self {
        Self { results: vec![], result_type: ResultType::List, searcher: String::new() }
    }
}

#[derive(Debug, Clone)]
pub enum ActionData {
    None,
    LaunchApp { executable: String, args: Vec<String> },
    OpenUrl { url: String },
    CopyToClipboard { text: String },
    CopyImageToClipboard { base64_png: String, width: u32, height: u32 },
    CopyImageFile { path: String },
    RunFunction { function_name: String, params: Vec<String> },
    ShellCommand { command: String },
    RunScript { path: String },
    OpenInTerminal { path: String },
    RofiSelect { name: String, response_socket: String },
}

impl ActionData {
    pub fn stable_id(&self) -> String {
        match self {
            ActionData::LaunchApp { executable, .. } => format!("app:{}", executable),
            ActionData::OpenUrl { url } => format!("url:{}", url),
            ActionData::CopyToClipboard { text } => format!("copy:{:x}", djb2(text)),
            ActionData::CopyImageToClipboard { width, height, .. } => {
                format!("copy-img:{}x{}", width, height)
            }
            ActionData::CopyImageFile { path } => format!("copy-file:{}", path),
            ActionData::RunFunction { function_name, params } => {
                format!("fn:{}:{}", function_name, params.join(","))
            }
            ActionData::ShellCommand { command } => format!("shell:{}", command),
            ActionData::RunScript { path } => format!("script:{}", path),
            ActionData::OpenInTerminal { path } => format!("terminal:{}", path),
            ActionData::RofiSelect { .. } => "rofi:select".to_string(),
            ActionData::None => "none".to_string(),
        }
    }
}

fn djb2(s: &str) -> u64 {
    s.bytes()
        .fold(5381u64, |h, b| h.wrapping_mul(33).wrapping_add(b as u64))
}
