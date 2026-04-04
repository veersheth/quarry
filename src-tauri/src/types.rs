use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ResultItem {
    pub name: String,
    pub action_id: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    /// For image clipboard entries: base64 PNG thumbnail for the UI to render
    pub thumbnail: Option<String>,
    #[serde(skip)]
    pub action: ActionData,
}

impl ResultItem {
    pub fn new(name: impl Into<String>, action: ActionData) -> Self {
        Self {
            name: name.into(),
            action_id: String::new(),
            description: None,
            icon: None,
            thumbnail: None,
            action,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub results: Vec<ResultItem>,
    pub result_type: ResultType,
}

#[derive(Debug, Clone)]
pub enum ActionData {
    None,
    LaunchApp { executable: String, args: Vec<String> },
    OpenUrl { url: String },
    CopyToClipboard { text: String },
    /// Copies a full-resolution PNG (as base64 data URI) back to the system clipboard
    CopyImageToClipboard { base64_png: String, width: u32, height: u32 },
    RunFunction { function_name: String, params: Vec<String> },
    ShellCommand { command: String },
}
