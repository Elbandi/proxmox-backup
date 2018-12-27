use failure::*;

use std::path::{PathBuf, Path};
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::{Mutex, Arc};

use crate::config::datastore;
use super::chunk_store::*;
use super::image_index::*;

pub struct DataStore {
    chunk_store: ChunkStore,
    gc_mutex: Mutex<bool>,
}

lazy_static!{
    static ref datastore_map: Mutex<HashMap<String, Arc<DataStore>>> =  Mutex::new(HashMap::new());
}

impl DataStore {

    pub fn lookup_datastore(name: &str) -> Result<Arc<DataStore>, Error> {

        let config = datastore::config()?;
        let (_, store_config) = config.sections.get(name)
            .ok_or(format_err!("no such datastore '{}'", name))?;

        let path = store_config["path"].as_str().unwrap();

        let mut map = datastore_map.lock().unwrap();

        if let Some(datastore) = map.get(name) {
            // Compare Config - if changed, create new Datastore object!
            if datastore.chunk_store.base == PathBuf::from(path) {
                return Ok(datastore.clone());
            }
        }

        if let Ok(datastore) = DataStore::open(name)  {
            let datastore = Arc::new(datastore);
            map.insert(name.to_string(), datastore.clone());
            return Ok(datastore);
        }

        bail!("store not found");
    }

    pub fn open(store_name: &str) -> Result<Self, Error> {

        let config = datastore::config()?;
        let (_, store_config) = config.sections.get(store_name)
            .ok_or(format_err!("no such datastore '{}'", store_name))?;

        let path = store_config["path"].as_str().unwrap();

        let chunk_store = ChunkStore::open(store_name, path)?;

        Ok(Self {
            chunk_store: chunk_store,
            gc_mutex: Mutex::new(false),
        })
    }

    pub fn create_image_writer<P: AsRef<Path>>(&self, filename: P, size: usize, chunk_size: usize) -> Result<ImageIndexWriter, Error> {

        let index = ImageIndexWriter::create(&self.chunk_store, filename.as_ref(), size, chunk_size)?;

        Ok(index)
    }

    pub fn open_image_reader<P: AsRef<Path>>(&self, filename: P) -> Result<ImageIndexReader, Error> {

        let index = ImageIndexReader::open(&self.chunk_store, filename.as_ref())?;

        Ok(index)
    }

    pub fn list_images(&self) -> Result<Vec<PathBuf>, Error> {
        let base = self.chunk_store.base_path();

        let mut list = vec![];

        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "iidx" {
                        list.push(path);
                    }
                }
            }
        }

        Ok(list)
    }

    fn mark_used_chunks(&self, status: &mut GarbageCollectionStatus) -> Result<(), Error> {

        let image_list = self.list_images()?;

        for path in image_list {
            let index = self.open_image_reader(path)?;
            index.mark_used_chunks(status)?;
        }

        Ok(())
   }

    pub fn garbage_collection(&self) -> Result<(), Error> {

        if let Ok(ref mut _mutex) = self.gc_mutex.try_lock() {

            let mut gc_status = GarbageCollectionStatus::default();
            gc_status.used_bytes = 0;

            println!("Start GC phase1 (mark chunks)");

            self.mark_used_chunks(&mut gc_status)?;

            println!("Start GC phase2 (sweep unused chunks)");
            self.chunk_store.sweep_used_chunks(&mut gc_status)?;

            println!("Used bytes: {}", gc_status.used_bytes);
            println!("Used chunks: {}", gc_status.used_chunks);
            println!("Disk bytes: {}", gc_status.disk_bytes);
            println!("Disk chunks: {}", gc_status.disk_chunks);

        } else {
            println!("Start GC failed - (already running/locked)");
        }

        Ok(())
    }
}
