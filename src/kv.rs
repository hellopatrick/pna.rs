use crate::command::{Command, Remove, Set};
use crate::errors::KVError;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, KVError>;

const REDUNDANCY_LIMIT: usize = 2048;

#[derive(Debug, Clone, Copy)]
struct Lookup {
  cask: usize,
  offset: u64,
}

pub struct KvStore {
  path: PathBuf,
  active_cask: usize,
  readers: HashMap<usize, io::BufReader<fs::File>>,
  writer: io::BufWriter<fs::File>,
  lookup: HashMap<String, Lookup>,
  current_redundancy: usize,
}

impl KvStore {
  fn current_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("current")
  }

  fn next_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().join("next")
  }

  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
    let path = path.as_ref();

    let current_dir = Self::current_dir(path);

    fs::create_dir_all(&current_dir)?;

    let mut cask_paths: Vec<_> = fs::read_dir(&current_dir)?
      .flatten()
      .map(|res| res.path())
      .filter(|p| is_cask(p))
      .collect();

    cask_paths.sort_unstable();

    let mut lookup = HashMap::new();
    let mut readers = HashMap::new();

    let mut current_redundancy = 0;

    for (i, path) in cask_paths.iter().enumerate() {
      let file = fs::File::open(path).expect("readable file");
      let mut rdr = io::BufReader::new(file);

      let mut offset = 0;

      while let Ok(cmd) = bincode::deserialize_from(&mut rdr) {
        match cmd {
          Command::Set(s) => {
            let key = s.key;

            let loc = Lookup { cask: i, offset };

            if let Some(_) = lookup.insert(key, loc) {
              current_redundancy += 1;
            }

            offset = rdr.seek(io::SeekFrom::Current(0))?;
          }
          Command::Rm(r) => {
            if let Some(_) = lookup.remove(&r.key) {
              current_redundancy += 1;
            }
          }
        }
      }

      readers.insert(i, rdr);
    }

    let active_cask = cask_paths.len();

    let active_cask_path = cask_path(&current_dir, active_cask);

    let active_writer_file = fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open(&active_cask_path)?;

    let active_writer = io::BufWriter::new(active_writer_file);

    let active_reader_file = fs::File::open(&active_cask_path)?;
    let active_reader = io::BufReader::new(active_reader_file);

    readers.insert(active_cask, active_reader);

    Ok(Self {
      path: path.to_path_buf(),
      readers,
      writer: active_writer,
      lookup,
      active_cask,
      current_redundancy,
    })
  }

  fn compact(&mut self) -> Result<()> {
    if self.current_redundancy < REDUNDANCY_LIMIT {
      return Ok(());
    }

    let next_dir = Self::next_dir(&self.path);
    fs::create_dir_all(&next_dir)?;

    let new_cask = 0;
    let new_cask_path = cask_path(&next_dir, new_cask);

    let next_cask_file = fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open(&new_cask_path)?;

    let mut next_writer = io::BufWriter::new(next_cask_file);

    for (_, loc) in self.lookup.iter() {
      let rdr = self.readers.get_mut(&loc.cask).expect("reader");
      rdr.seek(io::SeekFrom::Start(loc.offset))?;

      let v = bincode::deserialize_from(rdr)?;

      match &v {
        Command::Set(_) => bincode::serialize_into(&mut next_writer, &v)?,
        _ => (),
      };
    }

    let current_dir = Self::current_dir(&self.path);
    fs::remove_dir_all(&current_dir)?;
    fs::rename(&next_dir, &current_dir)?;

    self.current_redundancy = 0;

    self.build_index()?;

    Ok(())
  }

  fn build_index(&mut self) -> Result<()> {
    let new = Self::open(&self.path)?;

    self.writer = new.writer;
    self.readers = new.readers;
    self.lookup = new.lookup;
    self.active_cask = new.active_cask;
    self.current_redundancy = new.current_redundancy;

    Ok(())
  }

  pub fn get(&mut self, key: String) -> Result<Option<String>> {
    if let Some(loc) = self.lookup.get(&key) {
      if let Some(rdr) = self.readers.get_mut(&loc.cask) {
        rdr.seek(io::SeekFrom::Start(loc.offset))?;

        let v = bincode::deserialize_from(rdr)?;

        return match v {
          Command::Set(s) => Ok(Some(s.value)),
          _ => Ok(None),
        };
      }
    }

    Ok(None)
  }

  pub fn set(&mut self, key: String, value: String) -> Result<()> {
    let cmd = Command::Set(Set {
      key: key.clone(),
      value: value.clone(),
    });

    let offset = self.writer.seek(io::SeekFrom::Current(0))?;

    bincode::serialize_into(&mut self.writer, &cmd)?;

    if let Some(_) = self.lookup.insert(
      key,
      Lookup {
        cask: self.active_cask,
        offset,
      },
    ) {
      self.current_redundancy += 1;
    }

    self.writer.flush()?;

    self.compact()?;

    return Ok(());
  }

  pub fn remove(&mut self, key: String) -> Result<()> {
    if let Some(_) = self.lookup.remove(&key) {
      let cmd = Command::Rm(Remove { key });

      bincode::serialize_into(&mut self.writer, &cmd)?;

      self.writer.flush()?;

      self.current_redundancy += 1;

      self.compact()?;

      return Ok(());
    }

    Err(KVError::KeyNotFound(key))
  }
}

fn is_cask<P: AsRef<Path>>(path: P) -> bool {
  let path = path.as_ref();
  path.is_file() && path.extension() == Some("cask".as_ref())
}

fn cask_path<P: AsRef<Path>>(cask_dir: P, generation: usize) -> PathBuf {
  cask_dir.as_ref().join(format!("{:05}.cask", generation))
}
