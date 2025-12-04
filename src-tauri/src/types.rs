use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultItem {
    pub name: String,
    pub action_id: String,  
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub enum ResultType {
    List,
    Grid,
    Dictionary,
    Clipboard,
    ColorPicker
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub results: Vec<ResultItem>,
    pub result_type: ResultType,
}

// Action data that gets stored with the action_id
#[derive(Debug, Clone)]
pub enum ActionData {
    LaunchApp { executable: String, args: Vec<String> },
    OpenUrl { url: String },
    CopyToClipboard { text: String },
    RunFunction { function_name: String, params: Vec<String> },
    ShellCommand { command: String },
}
