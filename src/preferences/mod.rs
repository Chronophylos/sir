use anyhow::{ensure, Context, Result};
use ron::{de::from_reader, ser::to_writer};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Preferences {
    src_path: String,
    src_sheet: String,
    src_column: String,
    dest_path: String,
}

impl Preferences {
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

impl Default for Preferences {
    fn default() -> Self {
        Self {
            src_path: String::from(""),
            src_sheet: String::from(""),
            src_column: String::from(""),
            dest_path: String::from(""),
        }
    }
}

pub fn load_preferences() -> Result<Preferences> {
    todo!()
}

pub fn store_preferences(prefs: Preferences) -> Result<()> {
    todo!()
}
