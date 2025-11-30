#[derive(Debug, serde::Serialize, Clone)]
pub struct ResultItem {
    pub name: String,
    pub exec: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub enum ResultType {
    List,
    Grid,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct SearchResult {
    pub results: Vec<ResultItem>,
    pub result_type: ResultType,
}

