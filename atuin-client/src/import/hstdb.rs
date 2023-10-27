// import old shell history from hstdb!

use std::ffi::OsStr;
use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use eyre::Result;
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::Importer;
use crate::history::History;
use crate::import::Loader;

#[derive(Debug, Deserialize)]
pub struct HstdbEntry {
    pub time_finished: DateTime<Utc>,
    pub time_start: DateTime<Utc>,
    pub hostname: String,
    pub command: String,
    pub pwd: PathBuf,
    pub result: u16,
    pub session_id: Uuid,
    pub user: String,
}

fn convert_to_offset_datetime(
    date_time: chrono::DateTime<Utc>,
) -> Result<OffsetDateTime, eyre::Error> {
    let unix_timestamp = date_time.timestamp();
    Ok(OffsetDateTime::from_unix_timestamp(unix_timestamp)?)
}

impl From<HstdbEntry> for History {
    fn from(entry: HstdbEntry) -> Self {
        let timestamp = convert_to_offset_datetime(entry.time_start).unwrap();
        let command = entry.command;
        let cwd = entry.pwd.to_str().unwrap().to_string();
        let duration = (entry.time_finished - entry.time_start)
            .num_nanoseconds()
            .unwrap();
        let hostname = entry.hostname;
        let exit = entry.result as i64;

        let imported = History::import()
            .timestamp(timestamp)
            .command(command)
            .cwd(cwd)
            .duration(duration)
            .hostname(hostname)
            .exit(exit);

        imported.build().into()
    }
}

#[derive(Debug)]
pub struct Hstdb {
    entries: Vec<HstdbEntry>,
}

#[async_trait]
impl Importer for Hstdb {
    const NAME: &'static str = "hstdb";

    async fn new() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "hstdb", "hstdb").expect("can not get project dirs");
        let files = std::fs::read_dir(dirs.data_dir()).expect("can not read files from data dir");

        let mut entries = Vec::new();

        for file in files {
            let Ok(path) = file else { panic!("{file:?}") };

            if path.path().extension() != Some(OsStr::new("csv")) {
                continue;
            }

            let mut reader = csv::Reader::from_path(path.path())?;
            let mut new_entries: Vec<HstdbEntry> = reader
                .deserialize::<HstdbEntry>()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            entries.append(&mut new_entries);
        }

        Ok(Self { entries })
    }

    async fn entries(&mut self) -> Result<usize> {
        Ok(self.entries.len())
    }

    async fn load(self, h: &mut impl Loader) -> Result<()> {
        for entry in self.entries {
            h.push(entry.into()).await?;
        }
        Ok(())
    }
}
