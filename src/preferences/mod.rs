use crate::get_proj_dirs;
use anyhow::{ensure, Context, Result};
use log::{info, warn};
use ron::{de::from_reader, ser::to_writer};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PreferenceError {
    #[error("Could not get path to preference file")]
    NoPath,
    #[error("Preference file does not exist")]
    FileNotFound,
    #[error("Could not open preference file at {0}")]
    OpenFile(PathBuf),
    #[error("Could not deserialize preferences")]
    Deserialize,
    #[error("Could not serialize preferences")]
    Serialize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Preferences {
    pub src_path: String,
    pub src_sheet: String,
    pub src_column: String,
    pub dest_path: String,
}

impl Preferences {
    fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        ensure!(path.exists(), PreferenceError::FileNotFound);

        let file = File::open(path).with_context(|| PreferenceError::OpenFile(path.to_owned()))?;
        let reader = BufReader::new(file);

        Ok(from_reader(reader).context(PreferenceError::Deserialize)?)
    }

    fn write<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(path).context("Could not create preference file")?;
        let writer = BufWriter::new(file);

        to_writer(writer, self).context(PreferenceError::Serialize)?;

        Ok(())
    }
}

fn get_preferences_path() -> Result<PathBuf> {
    let dirs = get_proj_dirs()?;
    Ok(dirs.config_dir().join("preferences.ron"))
}

pub async fn load_preferences() -> Result<Preferences> {
    let path = get_preferences_path().context(PreferenceError::NoPath)?;
    info!("Loading preferences from `{}`", path.display());

    if !path.exists() {
        warn!("Using default values since preference file does not exist");
        return Ok(Preferences::default());
    }

    Preferences::from_path(path)
}

pub async fn store_preferences(prefs: Preferences) -> Result<()> {
    let path = get_preferences_path().context(PreferenceError::NoPath)?;

    let dir = path.parent().context("Preference file has no parent")?;
    if !dir.exists() {
        warn!(
            "Preference directory does not exist. Creating {}",
            dir.display()
        );
        create_dir_all(dir).context("Could not create directories for preferences")?;
    }

    info!("Storing preferences to `{}`", path.display());
    prefs
        .write(path.clone())
        .with_context(|| format!("Could not store preferences at `{}`", path.display()))
}
