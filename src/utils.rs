use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use crate::web_api::VersionInfo;

/// Converts the list of versions to a tree for easy version lookup
pub fn convert_version_list_to_tree(
    version_infos: Vec<VersionInfo>,
) -> HashMap<u64, HashMap<u64, HashMap<u64, VersionInfo>>> {
    let mut version_map = HashMap::new();

    for info in version_infos {
        let major_map = version_map.get_mut_or_insert(info.version.major, HashMap::new());
        let minor_map = major_map.get_mut_or_insert(info.version.minor, HashMap::new());
        minor_map.insert(info.version.patch, info);
    }

    version_map
}

trait GetOrInsert<K: Copy, V> {
    fn get_mut_or_insert(&mut self, key: K, default_value: V) -> &mut V;
}

impl<K: Eq + Hash + Copy, V> GetOrInsert<K, V> for HashMap<K, V> {
    fn get_mut_or_insert(&mut self, key: K, default_value: V) -> &mut V {
        if !self.contains_key(&key) {
            self.insert(key, default_value);
        }
        self.get_mut(&key).unwrap()
    }
}
