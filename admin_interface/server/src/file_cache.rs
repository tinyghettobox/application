use std::collections::HashMap;
use std::fs::{create_dir_all, read, remove_file, write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

pub struct InnerCache {
    pub(crate) cached_files: HashMap<String, SystemTime>,
    pub(crate) cache_folder: PathBuf,
    pub(crate) max_lifetime: Duration,
}

impl InnerCache {
    pub fn get(&mut self, name: String) -> Result<Vec<u8>, String> {
        if self.cached_files.get(&name).is_none() {
            return Err(format!("no uploaded file found for {}", name));
        }

        let file_path = self.cache_folder.join(name.clone());
        read(file_path).map_err(|error| format!("couldn't read cached file: {}", error))
    }

    pub fn add(&mut self, name: String, file: Vec<u8>) -> Result<(), String> {
        let file_path = self.cache_folder.join(name.clone());
        write(file_path, file).map_err(|error| format!("could not write cache file: {}", error))?;
        self.cached_files.insert(name, SystemTime::now());
        Ok(())
    }

    pub fn remove(&mut self, name: String) -> Result<(), String> {
        let file_path = self.cache_folder.join(name.clone());
        if file_path.exists() {
            remove_file(file_path.clone())
                .map_err(|error| format!("could not delete file '{:?}': {}", file_path, error))?;
        }
        self.cached_files.remove(&name);
        Ok(())
    }
}

#[derive(Clone)]
pub struct FileCache {
    inner: Arc<Mutex<InnerCache>>,
}

impl FileCache {
    pub fn new(cache_folder: String, max_lifetime: Duration) -> Self {
        info!("Checking cache file lifetimessss");
        let path = Path::new(&cache_folder);
        if !path.exists() {
            create_dir_all(&path).expect("could not create cache dir");
        }

        let cache = FileCache {
            inner: Arc::new(Mutex::new(InnerCache {
                cached_files: Default::default(),
                cache_folder: path.to_path_buf(),
                max_lifetime,
            })),
        };
        cache.start_cache_cleanup_timer();
        cache
    }

    pub fn get(&self, name: String) -> Result<Vec<u8>, String> {
        self.inner.lock().expect("couldn't lock").get(name)
    }

    pub fn add(&self, name: String, file: Vec<u8>) -> Result<(), String> {
        self.inner.lock().expect("couldn't lock").add(name, file)
    }

    fn start_cache_cleanup_timer(&self) {
        let cache = self.inner.clone();
        std::thread::spawn(move || loop {
            sleep(Duration::from_secs(60));
            info!("Checking cache file lifetimes");

            let mut cache = cache.lock().expect("couldn't lock");
            for (name, created) in cache.cached_files.clone() {
                let lifetime = SystemTime::now().duration_since(created).expect("could not check duration");
                if lifetime > cache.max_lifetime {
                    if let Err(error) = cache.remove(name) {
                        warn!("Could not clean cached file: {}", error);
                    }
                }
            }
        });
    }
}
