use bytes::Bytes;
use egui::TextureId;
use image::{self, GenericImageView, ImageFormat};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::{
    cell::UnsafeCell,
    collections::{hash_map::Entry, HashMap},
    sync::atomic::{AtomicBool, Ordering},
};

const TARGET: &str = "Image";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Key {
    Https(String),
}

pub struct LoadContext {
    loaded: AtomicBool,
    result: UnsafeCell<Result<TextureId, String>>,
}

impl Default for LoadContext {
    fn default() -> Self {
        Self {
            loaded: AtomicBool::new(false),
            result: UnsafeCell::new(Err(String::new())),
        }
    }
}

impl LoadContext {
    fn set_error(&self, e: impl Into<String>) {
        let ptr = self.result.get();
        unsafe {
            *ptr = Err(e.into());
        }
        self.loaded.store(true, Ordering::SeqCst);
    }
    fn set_texture_id(&self, id: TextureId) {
        let ptr = self.result.get();
        unsafe {
            *ptr = Ok(id);
        }
        self.loaded.store(true, Ordering::SeqCst);
    }

    pub fn get_texture_id(&self) -> Option<TextureId> {
        if self.loaded.load(Ordering::Relaxed) {
            unsafe { &*self.result.get() }.as_ref().ok().copied()
        } else {
            None
        }
    }
    pub fn get_error(&self) -> Option<&str> {
        if self.loaded.load(Ordering::Relaxed) {
            unsafe { &*self.result.get() }
                .as_ref()
                .err()
                .map(|s| s.as_str())
        } else {
            None
        }
    }
}

impl std::fmt::Debug for LoadContext {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("LoadContext")
            .field("loaded", &self.loaded)
            .finish_non_exhaustive()
    }
}

unsafe impl Sync for LoadContext {}

pub struct ToUIImage {
    key: Key,
    context: Arc<LoadContext>,
    image: epi::Image,
}

impl ToUIImage {
    pub fn finish_load(self, frame: &mut epi::Frame) {
        let texture = frame.alloc_texture(self.image);
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

pub async fn load_image_async(key: Key, context: Arc<LoadContext>) -> Option<ToUIImage> {
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
    static ref CACHE: Mutex<HashMap<Key, Arc<LoadContext>>> = Mutex::default();
}

pub fn get_context(key: Key) -> Arc<LoadContext> {
    let mut lock = CACHE.lock().unwrap();
    match lock.entry(key) {
        Entry::Occupied(o) => Arc::clone(o.get()),
        Entry::Vacant(v) => {
            let context = Arc::new(LoadContext::default());
            crate::background::start_loading_image(v.key().clone(), Arc::clone(&context));
            Arc::clone(v.insert(context))
        }
    }
}
