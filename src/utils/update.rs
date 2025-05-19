/*
 * Parseable Server (C) 2022 - 2024 Parseable, Inc.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

use super::uid;
use crate::about;
use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LatestRelease {
    pub version: semver::Version,
    pub date: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum ReleaseError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse version information from response")]
    VersionParseError,

    #[error("Missing or invalid published date in response")]
    DateParseError,

    #[error("Invalid date format: {0}")]
    DateFormatError(#[from] chrono::format::ParseError),
}

pub async fn get_latest(deployment_id: &uid::Uid) -> Result<LatestRelease, ReleaseError> {
    let agent = reqwest::ClientBuilder::new()
        .user_agent(about::user_agent(deployment_id))
        .timeout(Duration::from_secs(8))
        .build()?;
    let json: serde_json::Value = agent
        .get("https://download.parseable.io/latest-version")
        .send()
        .await?
        .json()
        .await?;
    let version = json["tag_name"]
        .as_str()
        .and_then(|ver| ver.strip_prefix('v'))
        .and_then(|ver| semver::Version::parse(ver).ok())
        .ok_or(ReleaseError::VersionParseError)?;
    let date = json["published_at"]
        .as_str()
        .ok_or(ReleaseError::DateParseError)?;

    let date = chrono::DateTime::parse_from_rfc3339(date)?.into();

    Ok(LatestRelease { version, date })
}
