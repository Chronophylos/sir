use crate::get_proj_dirs;
use anyhow::{ensure, Context, Result};
use log::{info, warn};
use ron::{de::from_reader, ser::to_writer};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Preferences<'a> {
    pub src_path: Cow<'a, str>,
    pub src_sheet: Cow<'a, str>,
    pub src_column: Cow<'a, str>,
    pub dest_path: Cow<'a, str>,
}

impl Preferences<'_> {
    fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        ensure!(path.as_ref().exists(), "Preference file does not exist");

        let file = File::open(path).context("Could not open preference file")?;
        let reader = BufReader::new(file);

        Ok(from_reader(reader).context("Could not deserialize preferences")?)
    }

    fn write<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(path).context("Could not create preference file")?;
        let writer = BufWriter::new(file);

        to_writer(writer, self).context("Could not serialize perferences")?;

        Ok(())
    }
}

fn get_preferences_path() -> Result<PathBuf> {
    let dirs = get_proj_dirs()?;
    Ok(dirs.config_dir().join("preferences.ron"))
}

pub fn load_preferences<'a>() -> Result<Preferences<'a>> {
    let path = get_preferences_path().context("Could not get path to preference file")?;
    info!("Loading preferences from file (path: {})", path.display());

    if !path.exists() {
        warn!("Using default values since preference file does not exist");
        return Ok(Preferences::default());
    }

    Preferences::from_path(path).context("Could not load preferences")
}

pub fn store_preferences(prefs: Preferences) -> Result<()> {
    let path = get_preferences_path().context("Could not get path to preference file")?;

    let dir = path.parent().context("Preference file has no parent")?;
    if !dir.exists() {
        warn!(
            "Preference directory does not exist. Creating {}",
            dir.display()
        );
        create_dir_all(dir).context("Could not create directories for preferences")?;
    }

    info!("Writing preferences to file (path: {})", path.display());
    prefs
        .write(path.clone())
        .with_context(|| format!("Could not store preferences (path: {})", path.display()))
}
