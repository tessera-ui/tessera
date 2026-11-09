//! On-demand resources owned by a UI session.
//!
//! Pass generated assets to components or retain a resource and observe its loading state.

use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
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

/// Where a resource's bytes come from.
#[derive(Clone)]
pub enum Source {
    /// A generated asset.
    Asset(Asset),
    /// A file path, or a URL on the web.
    Path(Arc<str>),
    /// Bytes in memory. Equal bytes share one resource.
    Bytes(Arc<[u8]>),
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
    source: Source,
    owner: StateOwner,
    state: State<LoadState<T>>,
}

impl<T: Send + Sync + 'static> std::fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Resource").field(&self.0.source).finish()
    }
}

impl<T: Send + Sync + 'static> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: Send + Sync + 'static> Eq for Resource<T> {}

impl<T: Clone + Send + Sync + 'static> Resource<T> {
    /// Requests a decoded resource in the active UI session.
    ///
    /// Reading and decoding run off the UI thread. Retain the returned handle
    /// while using the resource; live requests for the same source and decoder
    /// share one result, and completion invalidates the builds and measurements
    /// that observed it.
    pub fn load(source: impl Into<Source>, decode: fn(Arc<[u8]>) -> io::Result<T>) -> Self {
        let source = source.into();
        let cache = local_runtime_resource::<ResourceCache<T>>();
        let key = (source.clone(), decode as usize);
        let resource = {
            let mut entries = cache.0.borrow_mut();
            if let Some(entry) = entries.get(&key).and_then(Weak::upgrade) {
                return Resource(entry);
            }
            entries.retain(|_, entry| entry.strong_count() > 0);
            let owner = StateOwner::new();
            let state = owner.state_with_key((), || LoadState::Loading);
            let entry = Arc::new(ResourceEntry {
                source: source.clone(),
                owner,
                state,
            });
            entries.insert(key, Arc::downgrade(&entry));
            Resource(entry)
        };
        let state = resource.0.state;
        let label = format!("{source:?}");
        let finish = move |result: io::Result<T>| {
            state.set(match result {
                Ok(value) => LoadState::Ready(value),
                Err(error) => {
                    tracing::warn!(source = label, %error, "resource loading failed");
                    LoadState::Failed(Arc::new(error))
                }
            });
        };
        let dispatcher = resource.0.owner.dispatcher();
        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = source.read().await;
            if dispatcher.is_alive() {
                let result = result.and_then(decode);
                dispatcher.dispatch(move || finish(result));
            }
        });
        #[cfg(not(target_family = "wasm"))]
        {
            let in_flight = runtime_resource::<InFlight>();
            *in_flight.0.lock() += 1;
            let loading = Loading(in_flight);
            let _ = WORKERS.send(Box::new(move || {
                let _loading = loading;
                if dispatcher.is_alive() {
                    let result = source.read().and_then(decode);
                    dispatcher.dispatch(move || finish(result));
                }
            }));
        }
        resource
    }

    /// Observes loading state on the owning UI thread and tracks build or measurement.
    pub fn get(&self) -> LoadState<T> {
        self.0.state.get()
    }
}

struct ResourceCache<T: Send + Sync + 'static>(
    RefCell<HashMap<(Source, usize), Weak<ResourceEntry<T>>>>,
);

impl<T: Send + Sync + 'static> Default for ResourceCache<T> {
    fn default() -> Self {
        Self(RefCell::default())
    }
}

#[cfg(not(target_family = "wasm"))]
type Work = Box<dyn FnOnce() + Send>;

#[cfg(not(target_family = "wasm"))]
static WORKERS: std::sync::LazyLock<std::sync::mpsc::Sender<Work>> =
    std::sync::LazyLock::new(|| {
        let (sender, receiver) = std::sync::mpsc::channel::<Work>();
        let receiver = Arc::new(parking_lot::Mutex::new(receiver));
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get().clamp(2, 4));
        for _ in 0..threads {
            let receiver = Arc::clone(&receiver);
            std::thread::spawn(move || {
                loop {
                    let work = receiver.lock().recv();
                    match work {
                        Ok(work) => work(),
                        Err(_) => break,
                    }
                }
            });
        }
        sender
    });

#[cfg(not(target_family = "wasm"))]
#[derive(Default)]
struct InFlight(parking_lot::Mutex<usize>, parking_lot::Condvar);

#[cfg(not(target_family = "wasm"))]
struct Loading(Arc<InFlight>);

#[cfg(not(target_family = "wasm"))]
impl Drop for Loading {
    fn drop(&mut self) {
        *self.0.0.lock() -= 1;
        self.0.1.notify_all();
    }
}

#[cfg(all(any(test, feature = "testing"), not(target_family = "wasm")))]
pub(crate) fn in_flight() -> bool {
    *runtime_resource::<InFlight>().0.lock() > 0
}

/// Blocks until this session's requested resources have been delivered.
#[cfg(all(
    any(test, feature = "testing", feature = "headless"),
    not(target_family = "wasm")
))]
pub(crate) fn settle() {
    let in_flight = runtime_resource::<InFlight>();
    let mut count = in_flight.0.lock();
    while *count > 0 {
        in_flight.1.wait(&mut count);
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

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
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
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asset(asset) => asset.fmt(f),
            Self::Path(path) => path.fmt(f),
            Self::Bytes(bytes) => write!(f, "{} bytes", bytes.len()),
        }
    }
}

impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Asset(a), Self::Asset(b)) => a == b,
            (Self::Path(a), Self::Path(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => Arc::ptr_eq(a, b) || a == b,
            _ => false,
        }
    }
}
impl Eq for Source {}

impl Hash for Source {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Asset(asset) => asset.hash(state),
            Self::Path(path) => path.hash(state),
            Self::Bytes(bytes) => bytes.hash(state),
        }
    }
}

impl Source {
    #[cfg(not(target_family = "wasm"))]
    fn read(&self) -> io::Result<Arc<[u8]>> {
        match self {
            Self::Asset(Asset {
                bytes: Some(bytes), ..
            }) => Ok(Arc::from(*bytes)),
            #[cfg(target_os = "android")]
            Self::Asset(asset) => android::read(asset.path),
            #[cfg(not(target_os = "android"))]
            Self::Asset(asset) => native::path(asset.path)
                .and_then(std::fs::read)
                .map(Arc::from),
            Self::Path(path) => std::fs::read(&**path).map(Arc::from),
            Self::Bytes(bytes) => Ok(Arc::clone(bytes)),
        }
    }

    #[cfg(target_family = "wasm")]
    async fn read(&self) -> io::Result<Arc<[u8]>> {
        match self {
            Self::Asset(Asset {
                bytes: Some(bytes), ..
            }) => Ok(Arc::from(*bytes)),
            Self::Asset(asset) => web::read(asset.path).await,
            Self::Path(path) => web::read(path).await,
            Self::Bytes(bytes) => Ok(Arc::clone(bytes)),
        }
    }
}

macro_rules! source_from {
    ($($ty:ty => |$value:ident| $source:expr),* $(,)?) => {$(
        impl From<$ty> for Source {
            fn from($value: $ty) -> Self {
                $source
            }
        }
    )*};
}

source_from! {
    Asset => |asset| Self::Asset(asset),
    &str => |path| Self::Path(path.into()),
    String => |path| Self::Path(path.into()),
    Arc<str> => |path| Self::Path(path),
    &Path => |path| Self::Path(path.to_string_lossy().into()),
    PathBuf => |path| Self::Path(path.to_string_lossy().into()),
    &[u8] => |bytes| Self::Bytes(bytes.into()),
    Vec<u8> => |bytes| Self::Bytes(bytes.into()),
    Arc<[u8]> => |bytes| Self::Bytes(bytes),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delivered<T: Clone + Send + Sync + 'static>(resource: &Resource<T>) -> LoadState<T> {
        assert!(matches!(resource.get(), LoadState::Loading));
        settle();
        crate::dispatcher::flush();
        resource.get()
    }

    fn upper(bytes: Arc<[u8]>) -> io::Result<Arc<[u8]>> {
        assert!(std::thread::current().name().is_none());
        Ok(Arc::from(bytes.to_ascii_uppercase()))
    }

    #[cfg(not(any(target_family = "wasm", target_os = "android")))]
    #[test]
    fn sources_decode_off_the_ui_thread_and_share_live_results() {
        let file = tempfile::NamedTempFile::new().expect("new succeeds in the test fixture");
        std::fs::write(file.path(), b"file").expect("path succeeds in the test fixture");
        crate::RuntimeSession::new().run(|| {
            let asset = Asset::embedded("test/shared", b"original");
            let decoded = Resource::load(asset, upper);
            let LoadState::Ready(bytes) = delivered(&decoded) else {
                panic!("decoded asset")
            };
            assert_eq!(&*bytes, b"ORIGINAL");
            assert_eq!(decoded, Resource::load(asset, upper));
            let memory = Resource::load(b"bytes".to_vec(), upper);
            assert_eq!(memory, Resource::load(&b"bytes"[..], upper));
            assert!(matches!(delivered(&memory), LoadState::Ready(bytes) if &*bytes == b"BYTES"));
            let LoadState::Ready(bytes) = delivered(&Resource::load(file.path(), Ok)) else {
                panic!("file")
            };
            assert_eq!(&*bytes, b"file");
            assert!(matches!(
                delivered(&Resource::<()>::load(asset, |_| Err(io::Error::other(
                    "invalid"
                )))),
                LoadState::Failed(_)
            ));
        });
    }
}
