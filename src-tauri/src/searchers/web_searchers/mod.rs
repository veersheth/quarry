pub mod github;
pub mod google;
pub mod nix;
pub mod urls;
pub mod youtube;

pub use github::GithubSearcher;
pub use google::GoogleSearcher;
pub use nix::NixSearcher;
pub use urls::URLSearcher;
pub use youtube::YouTubeSearcher;
