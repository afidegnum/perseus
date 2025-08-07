use crate::state::{Freeze, MakeRx, MakeUnrx};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use sycamore::reactive::{create_signal, Signal};

/// A reactive version of [`HashMap`] that uses nested reactivity on its elements.
/// This requires nothing but `Clone + 'static` of the elements inside the map,
/// and it wraps them in `Signal`s to make them reactive. If you want to store
/// nested reactive types inside the map (e.g. `String`s), you should
/// use [`super::RxHashMapNested`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RxHashMap<K, V>(HashMap<K, V>)
where
    K: Clone + Eq + Hash,
    // We get the `Deserialize` derive macro working by tricking Serde by not
    // including the actual bounds here
    V: Clone + 'static;

/// The reactive version of [`RxHashMap`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RxHashMapRx<K, V>(Signal<HashMap<K, Signal<V>>>)
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Clone + Serialize + DeserializeOwned + 'static;

// --- Reactivity implementations ---
impl<K, V> MakeRx for RxHashMap<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    type Rx = RxHashMapRx<K, V>;

    fn make_rx(self) -> Self::Rx {
        RxHashMapRx(create_signal(
            self.0
                .into_iter()
                .map(|(k, v)| (k, create_signal(v)))
                .collect(),
        ))
    }
}

impl<K, V> MakeUnrx for RxHashMapRx<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    type Unrx = RxHashMap<K, V>;

    fn make_unrx(self) -> Self::Unrx {
        let map = self.0.get_clone();
        RxHashMap(map.into_iter().map(|(k, v)| (k, v.get_clone())).collect())
    }

    #[cfg(any(client, doc))]
    fn compute_suspense(&self) {}
}

// --- Dereferencing ---
impl<K, V> Deref for RxHashMap<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> Deref for RxHashMapRx<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    type Target = Signal<HashMap<K, Signal<V>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// --- Conversion implementation ---
impl<K, V> From<HashMap<K, V>> for RxHashMap<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    fn from(value: HashMap<K, V>) -> Self {
        Self(value)
    }
}

// --- Freezing implementation ---
impl<K, V> Freeze for RxHashMapRx<K, V>
where
    K: Clone + Serialize + DeserializeOwned + Eq + Hash,
    V: Clone + Serialize + DeserializeOwned + 'static,
{
    fn freeze(&self) -> String {
        let unrx = Self(self.0).make_unrx();
        // This should never panic, because we're dealing with a hashmap
        serde_json::to_string(&unrx).unwrap()
    }
}
