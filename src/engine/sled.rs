use crate::engine::KvsEngine;
use crate::Result;

pub struct Sled(sled::Db);

impl Sled {
    pub fn new(db: sled::Db) -> Sled {
        Self(db)
    }
}

impl KvsEngine for Sled {
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let tree: &sled::Tree = &self.0;
        tree.insert(key, value).map(|_| ())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&mut self, key: &str) -> Result<Option<String>> {
        let tree: &sled::Tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&mut self, key: &str) -> Result<Option<()>> {
        let tree: &sled::Tree = &self.0;
        if let None = tree.remove(key)? {
            return Ok(None);
        }
        tree.flush()?;
        Ok(Some(()))
    }
}
