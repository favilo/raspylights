use std::{borrow::Cow, error::Error, path::Path};

use async_std::task;
use heed::{types::Str, BytesDecode, BytesEncode, Env, EnvOpenOptions};
use lights::details::Details;
use serde::{Deserialize, Serialize};
use std::fs;

/// Get this working with MsgPack so the types are more stable than bincode.
struct SerdeMsgPack<T>(std::marker::PhantomData<T>);

impl<'a, T: 'a> BytesEncode<'a> for SerdeMsgPack<T>
where
    T: Serialize,
{
    type EItem = T;

    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, Box<dyn Error>> {
        Ok(Cow::Owned(rmp_serde::to_vec_named(item)?))
    }
}

impl<'a, T: 'a> BytesDecode<'a> for SerdeMsgPack<T>
where
    T: Deserialize<'a>,
{
    type DItem = T;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, Box<dyn Error>> {
        Ok(rmp_serde::from_read_ref(bytes)?)
    }
}

unsafe impl<T> Send for SerdeMsgPack<T> {}

unsafe impl<T> Sync for SerdeMsgPack<T> {}

pub(crate) struct Storage {
    env: Env,
    effect_database: heed::Database<Str, SerdeMsgPack<Details>>,
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
        let val = self
            .effect_database
            .get(&txn, key.as_ref())
            // If we can't load it for some reason, don't fret, it is probably
            // just an issue with a new version of the program...
            .unwrap_or(Default::default());
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
