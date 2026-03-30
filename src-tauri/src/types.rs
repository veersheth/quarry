use serde::{Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct ResultItem {
    pub name: String,
    pub action_id: String,          
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(skip)]                 
    pub action: ActionData,
}

impl ResultItem {
    pub fn new(
        name: impl Into<String>,
        action: ActionData,
    ) -> Self {
        Self {
            name: name.into(),
            action_id: String::new(), 
            description: None,
            icon: None,
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
    RunFunction { function_name: String, params: Vec<String> },
    ShellCommand { command: String },
}
