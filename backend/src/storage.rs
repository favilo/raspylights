use std::{error::Error, path::Path};

use async_std::task;
use heed::{types::*, Env, EnvOpenOptions};
use lights::details::Details;
use std::fs;

pub(crate) struct Storage {
    env: Env,
    effect_database: heed::Database<Str, SerdeBincode<Details>>,
}

impl Storage {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, heed::Error> {
        fs::create_dir_all(&path)?;
        let env = EnvOpenOptions::new().open(path)?;
        let effect_database = env.create_database(None)?;
        Ok(Self {
            env,
            effect_database,
        })
    }

    pub(crate) fn load<S>(&mut self, key: S) -> Result<Option<Details>, heed::Error>
    where
        S: AsRef<str>,
    {
        let txn = self.env.read_txn()?;
        let val = self.effect_database.get(&txn, key.as_ref())?;
        Ok(val)
    }

    pub(crate) async fn store(
        &mut self,
        key: &'static str,
        details: Details,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let env = self.env.clone();
        let db = self.effect_database.clone();
        let key = key.clone();
        task::spawn_blocking(move || {
            let mut txn = env
                .write_txn()
                .map_err(|_| lights::error::Error::HeedError)?;
            db.put(&mut txn, key, &details)
                .map_err(|_| lights::error::Error::HeedError)?;
            Ok(txn.commit().map_err(|_| lights::error::Error::HeedError)?)
        })
        .await
    }
}
