//! Collection of security materials (Keys, NID, AID, etc) used for encryption and decryption.
use crate::crypto::key::{
    AppKey, BeaconKey, DevKey, EncryptionKey, IdentityKey, NetKey, PrivacyKey,
};
use crate::crypto::{k2, KeyRefreshPhases, NetworkID, AID};
use crate::mesh::{AppKeyIndex, IVIndex, IVUpdateFlag, NetKeyIndex, NID};
use alloc::collections::btree_map;
use core::fmt::{Display, Error, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct NetworkKeys {
    nid: NID,
    encryption: EncryptionKey,
    privacy: PrivacyKey,
}
impl Display for NetworkKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "nid: {} encryption: {:X} privacy: {:X}",
            self.nid,
            self.encryption.key(),
            self.privacy.key()
        )
    }
}
impl NetworkKeys {
    pub fn new(nid: NID, encryption: EncryptionKey, privacy: PrivacyKey) -> Self {
        Self {
            nid,
            encryption,
            privacy,
        }
    }
    pub fn nid(&self) -> NID {
        self.nid
    }
    pub fn encryption_key(&self) -> &EncryptionKey {
        &self.encryption
    }
    pub fn privacy_key(&self) -> &PrivacyKey {
        &self.privacy
    }
}
impl From<&NetKey> for NetworkKeys {
    fn from(k: &NetKey) -> Self {
        let (nid, encryption, privacy) = k2(k.key(), b"\x00");
        Self::new(nid, encryption, privacy)
    }
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct NetworkSecurityMaterials {
    net_key: NetKey,
    network_keys: NetworkKeys,
    network_id: NetworkID,
    identity_key: IdentityKey,
    beacon_key: BeaconKey,
}
impl Display for NetworkSecurityMaterials {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "net_key: {:X} {} network_id: {} identity: {:X} beacon: {:X}",
            self.net_key.key(),
            &self.network_keys,
            self.network_id,
            self.identity_key.key(),
            self.beacon_key.key()
        )
    }
}
impl NetworkSecurityMaterials {
    pub fn net_key(&self) -> &NetKey {
        &self.net_key
    }
    pub fn network_keys(&self) -> &NetworkKeys {
        &self.network_keys
    }
    pub fn network_id(&self) -> NetworkID {
        self.network_id
    }
    pub fn identity_key(&self) -> &IdentityKey {
        &self.identity_key
    }
    pub fn beacon_key(&self) -> &BeaconKey {
        &self.beacon_key
    }
}
impl NetworkSecurityMaterials {}
impl From<&NetKey> for NetworkSecurityMaterials {
    fn from(k: &NetKey) -> Self {
        Self {
            net_key: *k,
            network_keys: k.into(),
            network_id: k.into(),
            identity_key: k.into(),
            beacon_key: k.into(),
        }
    }
}
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyPair<K: Clone + Copy + Eq + PartialEq> {
    pub new: K,
    pub old: K,
}
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum KeyPhase<K: Clone + Copy + Eq + PartialEq> {
    Normal(K),
    Phase1(KeyPair<K>),
    Phase2(KeyPair<K>),
}
impl<K: Clone + Copy + Eq> KeyPhase<K> {
    pub fn phase(&self) -> KeyRefreshPhases {
        match self {
            KeyPhase::Normal(_) => KeyRefreshPhases::Normal,
            KeyPhase::Phase1(_) => KeyRefreshPhases::First,
            KeyPhase::Phase2(_) => KeyRefreshPhases::Second,
        }
    }
    pub fn tx_key(&self) -> &K {
        match self {
            KeyPhase::Normal(k) => k,
            KeyPhase::Phase1(p) => &p.old,
            KeyPhase::Phase2(p) => &p.new,
        }
    }
    pub fn rx_keys(&self) -> (&K, Option<&K>) {
        match self {
            KeyPhase::Normal(k) => (k, None),
            KeyPhase::Phase1(p) => (&p.old, Some(&p.new)),
            KeyPhase::Phase2(p) => (&p.new, Some(&p.old)),
        }
    }
    pub fn key_pair(&self) -> Option<&KeyPair<K>> {
        match self {
            KeyPhase::Normal(_) => None,
            KeyPhase::Phase1(p) => Some(p),
            KeyPhase::Phase2(p) => Some(p),
        }
    }
}

#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct NetKeyMap {
    pub map: btree_map::BTreeMap<NetKeyIndex, KeyPhase<NetworkSecurityMaterials>>,
}
impl NetKeyMap {
    pub fn new() -> Self {
        Self {
            map: btree_map::BTreeMap::new(),
        }
    }

    /// Returns all `NetworkSecurityMaterials` matching `nid_to_match`. Because `NID` is a 7-bit value,
    /// one `NID` can match multiple different networks. For this reason, this functions returns an
    /// iterator that yields each matching network security materials. Only attempting to decrypt
    /// the Network PDU (and it failing/succeeding) will tell you if the `NID` and `NetworkKeys` match.
    pub fn matching_nid(
        &self,
        nid_to_match: NID,
    ) -> NIDFilterMap<btree_map::Iter<'_, NetKeyIndex, KeyPhase<NetworkSecurityMaterials>>> {
        NIDFilterMap {
            iter: self.map.iter(),
            nid: nid_to_match,
            temp: None,
        }
    }
    pub fn get_keys(&self, index: NetKeyIndex) -> Option<&KeyPhase<NetworkSecurityMaterials>> {
        self.map.get(&index)
    }
    pub fn get_keys_mut(
        &mut self,
        index: NetKeyIndex,
    ) -> Option<&mut KeyPhase<NetworkSecurityMaterials>> {
        self.map.get_mut(&index)
    }
    pub fn remove_keys(
        &mut self,
        index: NetKeyIndex,
    ) -> Option<KeyPhase<NetworkSecurityMaterials>> {
        self.map.remove(&index)
    }
    pub fn insert(
        &mut self,
        index: NetKeyIndex,
        new_key: &NetKey,
    ) -> Option<KeyPhase<NetworkSecurityMaterials>> {
        self.map.insert(index, KeyPhase::Normal(new_key.into()))
    }
}
pub struct NIDFilterMap<
    'a,
    I: Iterator<Item = (&'a NetKeyIndex, &'a KeyPhase<NetworkSecurityMaterials>)>,
> {
    iter: I,
    nid: NID,
    temp: Option<(NetKeyIndex, &'a NetworkSecurityMaterials)>,
}
impl<'a, I: Iterator<Item = (&'a NetKeyIndex, &'a KeyPhase<NetworkSecurityMaterials>)>> Iterator
    for NIDFilterMap<'a, I>
{
    type Item = (NetKeyIndex, &'a NetworkSecurityMaterials);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.temp.take() {
            return Some(v);
        }
        let (index, phase) = self.iter.next()?;
        match phase.rx_keys() {
            (first, Some(second))
                if first.network_keys.nid == self.nid && second.network_keys.nid == self.nid =>
            {
                self.temp = Some((*index, second));
                Some((*index, first))
            }
            (materials, None) if materials.network_keys.nid == self.nid => {
                Some((*index, materials))
            }
            (_, Some(materials)) if materials.network_keys.nid == self.nid => {
                Some((*index, materials))
            }
            _ => self.next(),
        }
    }
}
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationSecurityMaterials {
    pub app_key: AppKey,
    pub aid: AID,
    pub net_key_index: NetKeyIndex,
}
impl ApplicationSecurityMaterials {
    pub fn new(app_key: AppKey, net_key_index: NetKeyIndex) -> Self {
        Self {
            app_key,
            aid: app_key.aid(),
            net_key_index,
        }
    }
}
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct AppKeyMap {
    pub map: btree_map::BTreeMap<AppKeyIndex, ApplicationSecurityMaterials>,
}
impl AppKeyMap {
    pub fn new() -> Self {
        Self {
            map: btree_map::BTreeMap::new(),
        }
    }

    pub fn get_key(&self, index: AppKeyIndex) -> Option<&ApplicationSecurityMaterials> {
        self.map.get(&index)
    }
    pub fn get_key_mut(&mut self, index: AppKeyIndex) -> Option<&mut ApplicationSecurityMaterials> {
        self.map.get_mut(&index)
    }
    pub fn remove_key(&mut self, index: AppKeyIndex) -> Option<ApplicationSecurityMaterials> {
        self.map.remove(&index)
    }
    pub fn insert(
        &mut self,
        net_key_index: NetKeyIndex,
        app_key_index: AppKeyIndex,
        new_key: AppKey,
    ) -> Option<ApplicationSecurityMaterials> {
        self.map.insert(
            app_key_index,
            ApplicationSecurityMaterials::new(new_key, net_key_index),
        )
    }
    /// Returns all `ApplicationSecurityMaterials` matching `aid_to_match`. Because `AID` is a 6-bit value,
    /// one `AID` can match multiple different application keys. For this reason, this functions returns an
    /// iterator that yields each matching application security materials. Only attempting to decrypt
    /// the Application Payload (and it failing/succeeding) will tell you if the `AID` and
    /// `ApplicationSecurityMaterials` match.
    pub fn matching_aid(
        &self,
        aid_to_match: AID,
    ) -> impl Iterator<Item = (AppKeyIndex, &'_ ApplicationSecurityMaterials)> {
        self.map.iter().filter_map(move |(&index, materials)| {
            if materials.aid == aid_to_match {
                Some((index, materials))
            } else {
                None
            }
        })
    }
}

#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SecurityMaterials {
    pub iv_update_flag: IVUpdateFlag,
    pub iv_index: IVIndex,
    pub dev_key: DevKey,
    pub net_key_map: NetKeyMap,
    pub app_key_map: AppKeyMap,
}
