pub mod google;
pub mod nix;
pub mod urls;
pub mod youtube;
pub mod github;

pub use google::GoogleSearcher;
pub use nix::NixSearcher;
pub use urls::URLSearcher;
pub use youtube::YouTubeSearcher;
pub use github::GithubSearcher;
