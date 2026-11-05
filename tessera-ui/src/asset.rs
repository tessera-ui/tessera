//! On-demand resources owned by a UI session.
//!
//! Pass generated assets to components or retain a resource and observe its loading state.

use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    sync::{Arc, Weak},
};

use crate::{State, StateOwner, local_runtime_resource};

#[cfg(not(target_family = "wasm"))]
use crate::runtime_resource;

#[cfg(target_os = "android")]
mod android;
#[cfg(not(any(target_os = "android", target_family = "wasm")))]
mod native;
#[cfg(target_family = "wasm")]
mod web;

/// A resource that may still be loading.
#[derive(Clone, Debug)]
pub enum LoadState<T> {
    /// The resource has been requested.
    Loading,
    /// The resource is available.
    Ready(T),
    /// Loading or decoding failed.
    Failed(Arc<io::Error>),
}

/// A generated, immutable asset reference.
#[derive(Clone, Copy)]
pub struct Asset {
    path: &'static str,
    bytes: Option<&'static [u8]>,
}

/// A font resource and the Unicode ranges that request it as a fallback.
#[derive(Clone, Copy)]
pub struct FontAsset {
    /// Font data.
    pub asset: Asset,
    /// Family selected by text styles.
    pub family: &'static str,
    /// Inclusive Unicode ranges that trigger automatic fallback loading.
    pub fallback: &'static [(u32, u32)],
    /// Restricts requests to these supported ranges; empty means unrestricted.
    pub coverage: &'static [(u32, u32)],
}

/// A shared resource lifetime. Retain it while observing its loading state.
#[derive(Clone)]
pub struct Resource<T: Send + Sync + 'static>(Arc<ResourceEntry<T>>);

struct ResourceEntry<T: Send + Sync + 'static> {
    asset: Asset,
    owner: StateOwner,
    state: State<LoadState<T>>,
}

impl<T: Send + Sync + 'static> std::fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Resource").field(&self.0.asset).finish()
    }
}

impl<T: Send + Sync + 'static> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: Send + Sync + 'static> Eq for Resource<T> {}

impl<T: Clone + Send + Sync + 'static> Resource<T> {
    /// Observes loading state on the owning UI thread and tracks build or measurement.
    pub fn get(&self) -> LoadState<T> {
        self.0.state.get()
    }
}

struct AssetCache<T: Send + Sync + 'static>(
    RefCell<HashMap<(&'static str, usize), Weak<ResourceEntry<T>>>>,
);

impl<T: Send + Sync + 'static> Default for AssetCache<T> {
    fn default() -> Self {
        Self(RefCell::default())
    }
}

#[cfg(not(target_family = "wasm"))]
struct AssetWorker(std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>);

#[cfg(not(target_family = "wasm"))]
impl Default for AssetWorker {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
        std::thread::spawn(move || {
            for work in receiver {
                work();
            }
        });
        Self(sender)
    }
}

impl std::fmt::Debug for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.fmt(f)
    }
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Asset {}

impl std::hash::Hash for Asset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl Asset {
    /// Creates an asset backed by static bytes.
    #[doc(hidden)]
    pub const fn embedded(path: &'static str, bytes: &'static [u8]) -> Self {
        Self {
            path,
            bytes: Some(bytes),
        }
    }

    /// Creates an asset delivered with the application.
    #[doc(hidden)]
    pub const fn platform(path: &'static str) -> Self {
        Self { path, bytes: None }
    }

    /// Returns the asset's content-addressed package path.
    pub fn path(self) -> &'static str {
        self.path
    }

    /// Requests bytes in the active UI session.
    ///
    /// Retain the returned handle, for example with `remember`. Repeated live
    /// requests share a result. Completion invalidates dependent builds
    /// and measurements. Embedded resources are available immediately.
    pub fn load(self) -> Resource<Arc<[u8]>> {
        self.load_with(Ok)
    }

    /// Requests and caches a decoded resource in the active UI session.
    ///
    /// Retain the returned handle while using the resource. Results and failures
    /// are shared by asset, output type, and decoder until the last handle drops.
    /// Decoders may run on a background thread.
    pub fn load_with<T: Clone + Send + Sync + 'static>(
        self,
        decode: fn(Arc<[u8]>) -> io::Result<T>,
    ) -> Resource<T> {
        let cache = local_runtime_resource::<AssetCache<T>>();
        let key = (self.path, decode as usize);
        let resource = {
            let mut entries = cache.0.borrow_mut();
            if let Some(entry) = entries.get(&key).and_then(Weak::upgrade) {
                return Resource(entry);
            }
            entries.retain(|_, entry| entry.strong_count() > 0);
            let owner = StateOwner::new();
            let state = owner.state_with_key((), || LoadState::Loading);
            let entry = Arc::new(ResourceEntry {
                asset: self,
                owner,
                state,
            });
            entries.insert(key, Arc::downgrade(&entry));
            Resource(entry)
        };
        let state = resource.0.state;
        let finish = move |result: io::Result<T>| {
            state.set(match result {
                Ok(value) => LoadState::Ready(value),
                Err(error) => {
                    tracing::warn!(asset = self.path, %error, "resource loading failed");
                    LoadState::Failed(Arc::new(error))
                }
            });
        };
        if let Some(bytes) = self.bytes {
            finish(decode(Arc::from(bytes)));
        } else {
            let dispatcher = resource.0.owner.dispatcher();
            #[cfg(target_family = "wasm")]
            wasm_bindgen_futures::spawn_local(async move {
                let result = web::read(self.path).await;
                if dispatcher.is_alive() {
                    let result = result.and_then(decode);
                    dispatcher.dispatch(move || finish(result));
                }
            });
            #[cfg(not(target_family = "wasm"))]
            let _ = runtime_resource::<AssetWorker>().0.send(Box::new(move || {
                if !dispatcher.is_alive() {
                    return;
                }
                #[cfg(target_os = "android")]
                let result = android::read(self.path);
                #[cfg(not(target_os = "android"))]
                let result = native::path(self.path)
                    .and_then(std::fs::read)
                    .map(Arc::from);
                let result = result.and_then(decode);
                dispatcher.dispatch(move || finish(result));
            }));
        }
        resource
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_assets_share_results_within_their_session() {
        let asset = Asset::embedded("test/shared", b"original");
        crate::RuntimeSession::new().run(|| {
            let resource = asset.load();
            let LoadState::Ready(first) = resource.get() else {
                panic!("embedded asset")
            };
            let LoadState::Ready(second) = asset.load().get() else {
                panic!("cached asset")
            };
            assert!(Arc::ptr_eq(&first, &second));
            let LoadState::Ready(upper) = asset
                .load_with(|bytes| Ok(Arc::<[u8]>::from(bytes.to_ascii_uppercase())))
                .get()
            else {
                panic!("decoded asset")
            };
            assert_eq!(&*upper, b"ORIGINAL");
            let released = {
                let resource = Asset::embedded("test/released", b"temporary").load();
                let LoadState::Ready(bytes) = resource.get() else {
                    panic!("embedded asset")
                };
                Arc::downgrade(&bytes)
            };
            assert!(released.upgrade().is_none());
            assert!(matches!(
                asset
                    .load_with::<()>(|_| Err(io::Error::other("invalid resource")))
                    .get(),
                LoadState::Failed(_)
            ));
        });
    }

    #[cfg(not(any(target_family = "wasm", target_os = "android")))]
    #[test]
    fn platform_resources_deliver_results_on_the_owning_ui_thread() {
        let executable = std::env::current_exe().unwrap();
        let file = tempfile::NamedTempFile::new_in(executable.parent().unwrap()).unwrap();
        std::fs::write(file.path(), b"background bytes").unwrap();
        let path = Box::leak(
            file.path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .into_boxed_str(),
        );
        crate::RuntimeSession::new().run(|| {
            let resource = Asset::platform(path).load();
            let (finished, wait) = std::sync::mpsc::channel();
            runtime_resource::<AssetWorker>()
                .0
                .send(Box::new(move || finished.send(()).unwrap()))
                .unwrap();
            wait.recv().unwrap();
            assert!(matches!(resource.get(), LoadState::Loading));
            crate::dispatcher::flush();
            let LoadState::Ready(bytes) = resource.get() else {
                panic!("completed resource")
            };
            assert_eq!(&*bytes, b"background bytes");
        });
    }
}
