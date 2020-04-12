use std::marker::PhantomData;

/// Ignite cache
/// Interface for all the cache operations.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IgniteCache<K, V> {
    _a : PhantomData<K>,
    _b : PhantomData<V>,
}
//  {
//     id: i32,
//     name: String,
// }

// impl IgniteCache<K,V> {

// }
