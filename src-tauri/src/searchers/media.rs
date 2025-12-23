// Media searcher for Linux using D-Bus MPRIS
// Returns results where:
// - name = KEY (Title, Artist, Album, AlbumArt, PlayPause, Next, Previous, Stop)
// - description = VALUE (the actual data)

use super::SearchProvider;
use crate::types::{ActionData, ResultItem, ResultType, SearchResult};
use crate::ACTION_REGISTRY;
use std::process::Command;
use tauri::AppHandle;

pub struct MediaSearcher;

#[derive(Debug, Clone)]
struct MediaPlayer {
    name: String,
    bus_name: String,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    art_url: Option<String>,
    status: String, // Playing, Paused, Stopped
}

impl MediaSearcher {
    fn get_active_players() -> Vec<MediaPlayer> {
        let output = Command::new("dbus-send")
            .args(&[
                "--session",
                "--dest=org.freedesktop.DBus",
                "--type=method_call",
                "--print-reply",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus.ListNames",
            ])
            .output();

        let Ok(output) = output else {
            return vec![];
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut players = vec![];

        for line in stdout.lines() {
            if line.contains("org.mpris.MediaPlayer2.") {
                if let Some(bus_name) = line.split('"').nth(1).map(|s| s.to_string()) {
                    if let Some(player) = Self::get_player_info(&bus_name) {
                        players.push(player);
                    }
                }
            }
        }

        players
    }

    fn get_player_info(bus_name: &str) -> Option<MediaPlayer> {
        // Get player identity
        let identity_output = Command::new("dbus-send")
            .args(&[
                "--session",
                "--print-reply",
                &format!("--dest={}", bus_name),
                "/org/mpris/MediaPlayer2",
                "org.freedesktop.DBus.Properties.Get",
                "string:org.mpris.MediaPlayer2",
                "string:Identity",
            ])
            .output()
            .ok()?;

        let identity = String::from_utf8_lossy(&identity_output.stdout);
        let name = identity
            .lines()
            .find(|l| l.contains("variant"))
            .and_then(|l| l.split('"').nth(1))
            .unwrap_or("Unknown")
            .to_string();

        // Get playback status
        let status_output = Command::new("dbus-send")
            .args(&[
                "--session",
                "--print-reply",
                &format!("--dest={}", bus_name),
                "/org/mpris/MediaPlayer2",
                "org.freedesktop.DBus.Properties.Get",
                "string:org.mpris.MediaPlayer2.Player",
                "string:PlaybackStatus",
            ])
            .output()
            .ok()?;

        let status_str = String::from_utf8_lossy(&status_output.stdout);
        let status = status_str
            .lines()
            .find(|l| l.contains("variant"))
            .and_then(|l| l.split('"').nth(1))
            .unwrap_or("Unknown")
            .to_string();

        // Get metadata
        let metadata_output = Command::new("dbus-send")
            .args(&[
                "--session",
                "--print-reply",
                &format!("--dest={}", bus_name),
                "/org/mpris/MediaPlayer2",
                "org.freedesktop.DBus.Properties.Get",
                "string:org.mpris.MediaPlayer2.Player",
                "string:Metadata",
            ])
            .output()
            .ok()?;

        let metadata = String::from_utf8_lossy(&metadata_output.stdout);
        let title = Self::extract_metadata_field(&metadata, "xesam:title");
        let artist = Self::extract_metadata_field(&metadata, "xesam:artist");
        let album = Self::extract_metadata_field(&metadata, "xesam:album");
        let art_url = Self::extract_metadata_field(&metadata, "mpris:artUrl");

        Some(MediaPlayer {
            name,
            bus_name: bus_name.to_string(),
            title,
            artist,
            album,
            art_url,
            status,
        })
    }

    fn extract_metadata_field(metadata: &str, field: &str) -> Option<String> {
        let lines: Vec<&str> = metadata.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(field) {
                // Look for the value in subsequent lines
                for j in (i + 1)..lines.len().min(i + 10) {
                    let next_line = lines[j];
                    if next_line.contains("string") || next_line.contains("variant") {
                        if let Some(value) = next_line.split('"').nth(1) {
                            if !value.is_empty() {
                                return Some(value.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn create_media_control_results(player: &MediaPlayer, _query: &str) -> Vec<ResultItem> {
        let mut results = vec![];
        let bus_name = &player.bus_name;

        // 1. Title (KEY=Title, VALUE=song title)
        if let Some(title) = &player.title {
            let action_id = format!("media_title_{}", player.name.replace(" ", "_"));
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(action_id.clone(), ActionData::None);
            }
            results.push(ResultItem {
                name: "Title".to_string(),
                action_id,
                description: Some(title.clone()),
                icon: None,
            });
        }

        // 2. PlayPause (KEY=PlayPause, VALUE=current state)
        let play_state = if player.status == "Playing" {
            "Playing"
        } else {
            "Paused"
        };

        let action_id = format!("media_playpause_{}", player.name.replace(" ", "_"));
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::ShellCommand {
                    command: format!(
                        "dbus-send --session --dest={} --type=method_call /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.PlayPause",
                        bus_name
                    ),
                },
            );
        }
        results.push(ResultItem {
            name: "PlayPause".to_string(),
            action_id,
            description: Some(play_state.to_string()),
            icon: None,
        });

        // 3. Next (KEY=Next, VALUE=action description)
        let action_id = format!("media_next_{}", player.name.replace(" ", "_"));
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::ShellCommand {
                    command: format!(
                        "dbus-send --session --dest={} --type=method_call /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Next",
                        bus_name
                    ),
                },
            );
        }
        results.push(ResultItem {
            name: "Next".to_string(),
            action_id,
            description: Some("Skip to next track".to_string()),
            icon: None,
        });

        // 4. Previous (KEY=Previous, VALUE=action description)
        let action_id = format!("media_prev_{}", player.name.replace(" ", "_"));
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::ShellCommand {
                    command: format!(
                        "dbus-send --session --dest={} --type=method_call /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Previous",
                        bus_name
                    ),
                },
            );
        }
        results.push(ResultItem {
            name: "Previous".to_string(),
            action_id,
            description: Some("Go to previous track".to_string()),
            icon: None,
        });

        // 5. Stop (KEY=Stop, VALUE=action description)
        let action_id = format!("media_stop_{}", player.name.replace(" ", "_"));
        if let Ok(mut registry) = ACTION_REGISTRY.lock() {
            registry.register(
                action_id.clone(),
                ActionData::ShellCommand {
                    command: format!(
                        "dbus-send --session --dest={} --type=method_call /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Stop",
                        bus_name
                    ),
                },
            );
        }
        results.push(ResultItem {
            name: "Stop".to_string(),
            action_id,
            description: Some("Stop playback".to_string()),
            icon: None,
        });

        // 6. Artist (KEY=Artist, VALUE=artist name)
        if let Some(artist) = &player.artist {
            let action_id = format!("media_artist_{}", player.name.replace(" ", "_"));
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(action_id.clone(), ActionData::None);
            }
            results.push(ResultItem {
                name: "Artist".to_string(),
                action_id,
                description: Some(artist.clone()),
                icon: None,
            });
        }

        // 7. Album (KEY=Album, VALUE=album name)
        if let Some(album) = &player.album {
            let action_id = format!("media_album_{}", player.name.replace(" ", "_"));
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(action_id.clone(), ActionData::None);
            }
            results.push(ResultItem {
                name: "Album".to_string(),
                action_id,
                description: Some(album.clone()),
                icon: None,
            });
        }

        // 8. AlbumArt (KEY=AlbumArt, VALUE=art URL)
        if let Some(art_url) = &player.art_url {
            let action_id = format!("media_art_{}", player.name.replace(" ", "_"));
            if let Ok(mut registry) = ACTION_REGISTRY.lock() {
                registry.register(action_id.clone(), ActionData::None);
            }
            results.push(ResultItem {
                name: "AlbumArt".to_string(),
                action_id,
                description: Some(art_url.clone()),
                icon: None,
            });
        }

        results
    }
}

impl SearchProvider for MediaSearcher {
    fn search(&self, query: &str, _app: &AppHandle) -> SearchResult {
        let players = Self::get_active_players();

        if players.is_empty() {
            return SearchResult {
                results: vec![ResultItem {
                    name: "No active media players found".to_string(),
                    action_id: "no_media".to_string(),
                    description: Some(
                        "Start a media player (Spotify, VLC, Firefox, etc.)".to_string(),
                    ),
                    icon: None,
                }],
                result_type: ResultType::List,
                usage_sorted: false,
                additional_info: None,
            };
        }

        let mut all_results = vec![];

        // Only show controls for the first active player (currently playing)
        if let Some(player) = players.first() {
            all_results.extend(Self::create_media_control_results(player, query));
        }

        SearchResult {
            results: all_results,
            result_type: ResultType::Media,
            usage_sorted: false,
            additional_info: None,
        }
    }
}
