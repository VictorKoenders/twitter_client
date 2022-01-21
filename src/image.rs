use bytes::Bytes;
use egui::TextureId;
use image::{self, GenericImageView, ImageFormat};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::cell::Cell;
use std::collections::{hash_map::Entry, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TARGET: &str = "Image";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Key {
    Https(String),
}

#[derive(Clone)]
pub struct LoadContext(Arc<Inner>);

impl Default for LoadContext {
    fn default() -> Self {
        Self(Arc::new(Inner {
            result: RwLock::default(),
            last_access: Cell::new(Instant::now()),
        }))
    }
}

impl LoadContext {
    fn accessed(&self) {
        self.0.last_access.set(Instant::now());
    }

    fn set_error(&self, e: impl Into<String>) {
        self.accessed();
        *self.0.result.write() = LoadingStatus::Error(e.into());
    }
    fn set_texture_id(&self, id: TextureId) {
        self.accessed();
        *self.0.result.write() = LoadingStatus::Loaded(id);
    }

    pub fn get_texture_id(&self) -> Option<TextureId> {
        self.accessed();
        self.0.result.read().as_texture()
    }

    pub fn get_error(&self) -> Option<String> {
        self.accessed();
        self.0.result.read().as_error()
    }
}

impl std::fmt::Debug for LoadContext {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut dbg = fmt.debug_struct("LoadContext");
        let read = self.0.result.read();
        if read.is_loading() {
            dbg.field("loading", &true);
        } else if let Some(id) = read.as_texture() {
            dbg.field("texture_id", &id);
        } else if let Some(e) = read.as_error() {
            dbg.field("error", &e);
        }

        dbg.finish_non_exhaustive()
    }
}

struct Inner {
    result: RwLock<LoadingStatus>,
    last_access: Cell<Instant>,
}

unsafe impl Sync for Inner {}

enum LoadingStatus {
    Loading,
    Loaded(TextureId),
    Error(String),
}

impl Default for LoadingStatus {
    fn default() -> Self {
        Self::Loading
    }
}

impl LoadingStatus {
    fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    fn as_error(&self) -> Option<String> {
        match self {
            Self::Error(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_texture(&self) -> Option<TextureId> {
        match self {
            Self::Loaded(id) => Some(*id),
            _ => None,
        }
    }
}
pub struct ToUIImage {
    key: Key,
    context: LoadContext,
    image: epi::Image,
}

impl ToUIImage {
    pub fn finish_load(self, frame: &mut epi::Frame) {
        let texture = frame.alloc_texture(self.image);
        log::info!(
            target: TARGET,
            "Id is {}",
            match &texture {
                TextureId::User(id) => id,
                _ => unreachable!(),
            }
        );
        self.context.set_texture_id(texture);
    }
}

impl std::fmt::Debug for ToUIImage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ToUIImage")
            .field("key", &self.key)
            .field("context", &self.context)
            .finish_non_exhaustive()
    }
}

pub async fn load_image_async(key: Key, context: LoadContext) -> Option<ToUIImage> {
    log::info!(target: TARGET, "Loading {:?}", key);
    let (bytes, format): (Bytes, Option<ImageFormat>) = match &key {
        Key::Https(url) => {
            let response = match reqwest::get(url).await {
                Ok(response) => response,
                Err(e) => {
                    context.set_error(format!("Could not connect to server: {:?}", e));
                    return None;
                }
            };
            match response.bytes().await {
                Ok(bytes) => (bytes, None),
                Err(e) => {
                    context.set_error(format!("Could not download image: {:?}", e));
                    return None;
                }
            }
        }
    };
    log::info!(
        target: TARGET,
        "Loaded {} bytes, format is {:?}",
        bytes.len(),
        format
    );
    let result = if let Some(format) = format {
        image::load_from_memory_with_format(&bytes, format)
    } else {
        image::load_from_memory(&bytes)
    };
    match result {
        Ok(image) => {
            log::info!(
                target: TARGET,
                "Size is {}x{}",
                image.width(),
                image.height()
            );
            let image = epi::Image::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                &image.to_rgba8(),
            );
            return Some(ToUIImage {
                context,
                key,
                image,
            });
        }
        Err(e) => {
            context.set_error(format!("Could not decode image: {:?}", e.to_string()));
        }
    }
    None
}

lazy_static! {
    static ref LAST_CLEANUP_TIME: Mutex<Option<Instant>> = Mutex::default();
    static ref CACHE: Mutex<HashMap<Key, LoadContext>> = Mutex::default();
}

pub fn get_context(key: Key) -> LoadContext {
    let mut lock = CACHE.lock();
    match lock.entry(key) {
        Entry::Occupied(o) => o.get().clone(),
        Entry::Vacant(v) => {
            let context = LoadContext::default();
            crate::background::start_loading_image(v.key().clone(), context.clone());
            v.insert(context).clone()
        }
    }
}

pub fn cleanup(frame: &epi::Frame) {
    let mut keys_to_remove = Vec::new();
    let mut write = CACHE.lock();
    for (key, ctx) in write.iter_mut() {
        if ctx.0.last_access.get().elapsed() > Duration::from_secs(60) {
            keys_to_remove.push(key.clone());
        }
    }

    for key in keys_to_remove {
        let val = write.remove(&key).unwrap();
        let inner = match Arc::try_unwrap(val.0) {
            Ok(inner) => inner,
            Err(_) => {
                log::warn!(
                    target: TARGET,
                    "Could not clean up texture; in use somewhere"
                );
                continue;
            }
        };
        let read = inner.result.read();
        if let Some(id) = read.as_texture() {
            log::debug!(target: TARGET, "Cleaning up {:?}", id);
            frame.free_texture(id);
        }
    }
}
