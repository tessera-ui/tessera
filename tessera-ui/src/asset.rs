//! On-demand resources owned by a UI session.
//!
//! Pass generated assets to components or retain a resource and observe its loading state.

use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    hash::{Hash, Hasher},
    io,
    ops::Deref,
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::{State, local_runtime_resource};

#[cfg(not(target_family = "wasm"))]
use crate::shared_runtime_resource;

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
    /// A generated asset packaged with the application.
    Asset(Asset),
    /// Bytes the application already holds. Equal bytes share one resource.
    Memory(Arc<[u8]>),
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

/// Bytes read from a [`Source`] and passed to a resource decoder.
///
/// Embedded assets are borrowed from the application binary instead of being
/// copied. Retaining the handle in a decoded value keeps the data available
/// without a second allocation.
#[derive(Clone)]
pub struct Bytes(BytesData);

#[derive(Clone)]
enum BytesData {
    Static(&'static [u8]),
    Shared(Arc<[u8]>),
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match &self.0 {
            BytesData::Static(bytes) => bytes,
            BytesData::Shared(bytes) => bytes,
        }
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.len())
    }
}

impl From<&'static [u8]> for Bytes {
    fn from(bytes: &'static [u8]) -> Self {
        Self(BytesData::Static(bytes))
    }
}

impl From<Arc<[u8]>> for Bytes {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self(BytesData::Shared(bytes))
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(BytesData::Shared(bytes.into()))
    }
}

/// A value produced asynchronously and observed by the UI.
///
/// Reading [`Resource::get`] during a build or measurement tracks the value;
/// completion invalidates those readers. Retain the handle while using the
/// value: dropping the last handle cancels unfinished work.
#[derive(Clone)]
pub struct Resource<T: 'static>(Rc<ResourceEntry<T>>);

struct ResourceEntry<T: 'static> {
    label: String,
    _owner: crate::runtime::ModelScope,
    state: State<LoadState<T>>,
    task: RefCell<Option<crate::Task>>,
}

impl<T> std::fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Resource").field(&self.0.label).finish()
    }
}

impl<T> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for Resource<T> {}

impl<T: Clone + Send + 'static> Resource<T> {
    /// Reads `source` and decodes it on [`crate::Dispatcher::Io`].
    ///
    /// Live requests for the same source and decoder share one result.
    pub fn load(source: impl Into<Source>, decode: fn(Bytes) -> io::Result<T>) -> Self {
        let source = source.into();
        let cache = local_runtime_resource::<ResourceCache<T>>();
        let key = (source.clone(), decode as usize);
        if let Some(entry) = cache.0.borrow().get(&key).and_then(Weak::upgrade) {
            return Resource(entry);
        }
        let resource = Self::pending(format!("{source:?}"));
        {
            let mut entries = cache.0.borrow_mut();
            entries.retain(|_, entry| entry.strong_count() > 0);
            entries.insert(key, Rc::downgrade(&resource.0));
        }
        #[cfg(target_family = "wasm")]
        resource.run_local(async move { source.read().await.and_then(decode) });
        #[cfg(not(target_family = "wasm"))]
        resource.run(async move { source.read().and_then(decode) });
        resource
    }

    /// Produces the value from `future` on [`crate::Dispatcher::Io`].
    pub fn spawn(future: impl Future<Output = io::Result<T>> + Send + 'static) -> Self {
        let resource = Self::pending(std::any::type_name::<T>().to_owned());
        resource.run(future);
        resource
    }

    /// Produces the value from `future` on the current UI session's thread.
    ///
    /// The future may hold values that cannot leave this thread. Move
    /// expensive computation to another dispatcher with [`crate::on`].
    pub fn spawn_local(future: impl Future<Output = io::Result<T>> + 'static) -> Self {
        let resource = Self::pending(std::any::type_name::<T>().to_owned());
        resource.run_local(future);
        resource
    }

    /// Returns the loading state and tracks it in the current build or
    /// measurement.
    pub fn get(&self) -> LoadState<T> {
        self.0.state.get()
    }

    fn pending(label: String) -> Self {
        let owner = crate::runtime::ModelScope::new();
        let state = owner.own_state(LoadState::Loading);
        Self(Rc::new(ResourceEntry {
            label,
            _owner: owner,
            state,
            task: RefCell::new(None),
        }))
    }

    fn deliverer(&self) -> impl FnOnce(io::Result<T>) + Send + 'static {
        let state = self.0.state;
        let session = crate::dispatcher::register_current();
        let label = self.0.label.clone();
        // Results return through the session queue so a UI thread waiting for
        // loads never blocks the delivery.
        move |result| {
            crate::dispatcher::dispatch_to_session(session, move || {
                if !state.is_alive() {
                    return;
                }
                state.set(match result {
                    Ok(value) => LoadState::Ready(value),
                    Err(error) => {
                        tracing::warn!(source = label, %error, "resource loading failed");
                        LoadState::Failed(Arc::new(error))
                    }
                });
            });
        }
    }

    fn run(&self, future: impl Future<Output = io::Result<T>> + Send + 'static) {
        let deliver = self.deliverer();
        #[cfg(not(target_family = "wasm"))]
        let loading = Loading::start();
        let task = crate::spawn(crate::Dispatcher::Io, async move {
            #[cfg(not(target_family = "wasm"))]
            let _loading = loading;
            deliver(future.await);
        });
        self.0.task.replace(Some(task));
    }

    fn run_local(&self, future: impl Future<Output = io::Result<T>> + 'static) {
        let deliver = self.deliverer();
        let task = crate::spawn_local(async move { deliver(future.await) });
        self.0.task.replace(Some(task));
    }
}

struct ResourceCache<T: 'static>(RefCell<HashMap<(Source, usize), Weak<ResourceEntry<T>>>>);

impl<T> Default for ResourceCache<T> {
    fn default() -> Self {
        Self(RefCell::default())
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Default)]
struct InFlight(parking_lot::Mutex<usize>, parking_lot::Condvar);

#[cfg(not(target_family = "wasm"))]
struct Loading(Arc<InFlight>);

#[cfg(not(target_family = "wasm"))]
impl Loading {
    fn start() -> Self {
        let in_flight = shared_runtime_resource::<InFlight>();
        *in_flight.0.lock() += 1;
        Self(in_flight)
    }
}

#[cfg(not(target_family = "wasm"))]
impl Drop for Loading {
    fn drop(&mut self) {
        *self.0.0.lock() -= 1;
        self.0.1.notify_all();
    }
}

#[cfg(all(any(test, feature = "testing"), not(target_family = "wasm")))]
pub(crate) fn in_flight() -> bool {
    *shared_runtime_resource::<InFlight>().0.lock() > 0
}

/// Blocks until this session's requested resources have been delivered.
#[cfg(all(
    any(test, feature = "testing", feature = "headless"),
    not(target_family = "wasm")
))]
pub(crate) fn settle() {
    let in_flight = shared_runtime_resource::<InFlight>();
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
            Self::Memory(bytes) => write!(f, "{} bytes", bytes.len()),
        }
    }
}

impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Asset(a), Self::Asset(b)) => a == b,
            (Self::Memory(a), Self::Memory(b)) => Arc::ptr_eq(a, b) || a == b,
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
            Self::Memory(bytes) => bytes.hash(state),
        }
    }
}

impl Source {
    #[cfg(not(target_family = "wasm"))]
    fn read(&self) -> io::Result<Bytes> {
        match self {
            Self::Asset(Asset {
                bytes: Some(bytes), ..
            }) => Ok((*bytes).into()),
            #[cfg(target_os = "android")]
            Self::Asset(asset) => android::read(asset.path).map(Bytes::from),
            #[cfg(not(target_os = "android"))]
            Self::Asset(asset) => native::path(asset.path)
                .and_then(std::fs::read)
                .map(Bytes::from),
            Self::Memory(bytes) => Ok(Arc::clone(bytes).into()),
        }
    }

    #[cfg(target_family = "wasm")]
    async fn read(&self) -> io::Result<Bytes> {
        match self {
            Self::Asset(Asset {
                bytes: Some(bytes), ..
            }) => Ok((*bytes).into()),
            Self::Asset(asset) => web::read(asset.path).await.map(Bytes::from),
            Self::Memory(bytes) => Ok(Arc::clone(bytes).into()),
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
    &[u8] => |bytes| Self::Memory(bytes.into()),
    Vec<u8> => |bytes| Self::Memory(bytes.into()),
    Arc<[u8]> => |bytes| Self::Memory(bytes),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delivered<T: Clone + Send + 'static>(resource: &Resource<T>) -> LoadState<T> {
        assert!(matches!(resource.get(), LoadState::Loading));
        settle();
        crate::dispatcher::flush();
        resource.get()
    }

    fn upper(bytes: Bytes) -> io::Result<Arc<[u8]>> {
        assert_eq!(std::thread::current().name(), Some("tessera-io"));
        Ok(Arc::from(bytes.to_ascii_uppercase()))
    }

    #[cfg(not(any(target_family = "wasm", target_os = "android")))]
    #[test]
    fn sources_decode_off_the_ui_thread_and_share_live_results() {
        crate::RuntimeSession::new().run(|| {
            static EMBEDDED: &[u8] = b"original";
            let asset = Asset::embedded("test/shared", EMBEDDED);
            let LoadState::Ready(address) =
                delivered(&Resource::load(asset, |bytes| Ok(bytes.as_ptr() as usize)))
            else {
                panic!("embedded bytes")
            };
            assert_eq!(address, EMBEDDED.as_ptr() as usize);
            let decoded = Resource::load(asset, upper);
            let LoadState::Ready(bytes) = delivered(&decoded) else {
                panic!("decoded asset")
            };
            assert_eq!(&*bytes, b"ORIGINAL");
            assert_eq!(decoded, Resource::load(asset, upper));
            let memory = Resource::load(b"bytes".to_vec(), upper);
            assert_eq!(memory, Resource::load(&b"bytes"[..], upper));
            assert!(matches!(delivered(&memory), LoadState::Ready(bytes) if &*bytes == b"BYTES"));
            assert!(matches!(
                delivered(&Resource::<()>::load(asset, |_| Err(io::Error::other(
                    "invalid"
                )))),
                LoadState::Failed(_)
            ));
        });
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn spawned_values_deliver_on_their_dispatchers_and_stop_with_their_last_handle() {
        struct Dropped(std::sync::mpsc::Sender<()>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }

        crate::RuntimeSession::new().run(|| {
            let background =
                Resource::spawn(async { Ok(std::thread::current().name().map(str::to_owned)) });
            assert!(matches!(
                delivered(&background),
                LoadState::Ready(Some(name)) if name == "tessera-io"
            ));

            let ui_thread = std::thread::current().id();
            let local = Rc::new(7);
            let value = Resource::spawn_local(async move {
                assert_eq!(std::thread::current().id(), ui_thread);
                Ok(*local)
            });
            assert!(matches!(value.get(), LoadState::Loading));
            crate::dispatcher::flush();
            crate::dispatcher::flush();
            assert!(matches!(value.get(), LoadState::Ready(7)));

            let (sender, dropped) = std::sync::mpsc::channel();
            let guard = Dropped(sender);
            let unfinished = Resource::<()>::spawn(async move {
                let _guard = guard;
                std::future::pending().await
            });
            drop(unfinished);
            dropped
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("dropping the last handle cancels the work");
        });
    }
}
