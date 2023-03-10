#[cfg(feature = "std")]
pub use impl_serde::serialize as bytes;
use scale::{Decode, Encode};
use scale_info::{
    build::{Fields, Variants},
    Path, Type, TypeInfo,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Consensus engine unique ID.
pub type ConsensusEngineId = [u8; 4];

/// Generic header digest.
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, TypeInfo))]
pub struct Digest {
    /// A list of logs in the digest.
    pub logs: Vec<DigestItem>,
}

impl Digest {
    /// Get reference to all digest items.
    #[allow(dead_code)]
    pub fn logs(&self) -> &[DigestItem] {
        &self.logs
    }

    /// Push new digest item.
    #[allow(dead_code)]
    pub fn push(&mut self, item: DigestItem) {
        self.logs.push(item);
    }

    /// Pop a digest item.
    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<DigestItem> {
        self.logs.pop()
    }

    /// Get reference to the first digest item that matches the passed predicate.
    #[allow(dead_code)]
    pub fn log<T: ?Sized, F: Fn(&DigestItem) -> Option<&T>>(&self, predicate: F) -> Option<&T> {
        self.logs().iter().find_map(predicate)
    }

    /// Get a conversion of the first digest item that successfully converts using the function.
    #[allow(dead_code)]
    pub fn convert_first<T, F: Fn(&DigestItem) -> Option<T>>(&self, predicate: F) -> Option<T> {
        self.logs().iter().find_map(predicate)
    }
}

/// Digest item that is able to encode/decode 'system' digest items and
/// provide opaque access to other items.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DigestItem {
    /// A pre-runtime digest.
    ///
    /// These are messages from the consensus engine to the runtime, although
    /// the consensus engine can (and should) read them itself to avoid
    /// code and state duplication. It is erroneous for a runtime to produce
    /// these, but this is not (yet) checked.
    ///
    /// NOTE: the runtime is not allowed to panic or fail in an `on_initialize`
    /// call if an expected `PreRuntime` digest is not present. It is the
    /// responsibility of a external block verifier to check this. Runtime API calls
    /// will initialize the block without pre-runtime digests, so initialization
    /// cannot fail when they are missing.
    PreRuntime(ConsensusEngineId, Vec<u8>),

    /// A message from the runtime to the consensus engine. This should *never*
    /// be generated by the native code of any consensus engine, but this is not
    /// checked (yet).
    Consensus(ConsensusEngineId, Vec<u8>),

    /// Put a Seal on it. This is only used by native code, and is never seen
    /// by runtimes.
    Seal(ConsensusEngineId, Vec<u8>),

    /// Some other thing. Unsupported and experimental.
    Other(Vec<u8>),

    /// An indication for the light clients that the runtime execution
    /// environment is updated.
    ///
    /// Currently this is triggered when:
    /// 1. Runtime code blob is changed or
    /// 2. `heap_pages` value is changed.
    RuntimeEnvironmentUpdated,
}

#[cfg(feature = "std")]
impl serde::Serialize for DigestItem {
    fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.using_encoded(|bytes| bytes::serialize(bytes, seq))
    }
}

#[cfg(feature = "std")]
impl<'a> serde::Deserialize<'a> for DigestItem {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let r = bytes::deserialize(de)?;
        Decode::decode(&mut &r[..])
            .map_err(|e| serde::de::Error::custom(format!("Decode error: {e}")))
    }
}

#[cfg(feature = "std")]
impl TypeInfo for DigestItem {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("DigestItem", module_path!()))
            .variant(
                Variants::new()
                    .variant("PreRuntime", |v| {
                        v.index(DigestItemType::PreRuntime as u8).fields(
                            Fields::unnamed()
                                .field(|f| {
                                    f.ty::<ConsensusEngineId>().type_name("ConsensusEngineId")
                                })
                                .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                        )
                    })
                    .variant("Consensus", |v| {
                        v.index(DigestItemType::Consensus as u8).fields(
                            Fields::unnamed()
                                .field(|f| {
                                    f.ty::<ConsensusEngineId>().type_name("ConsensusEngineId")
                                })
                                .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                        )
                    })
                    .variant("Seal", |v| {
                        v.index(DigestItemType::Seal as u8).fields(
                            Fields::unnamed()
                                .field(|f| {
                                    f.ty::<ConsensusEngineId>().type_name("ConsensusEngineId")
                                })
                                .field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                        )
                    })
                    .variant("Other", |v| {
                        v.index(DigestItemType::Other as u8).fields(
                            Fields::unnamed().field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")),
                        )
                    })
                    .variant("RuntimeEnvironmentUpdated", |v| {
                        v.index(DigestItemType::RuntimeEnvironmentUpdated as u8)
                            .fields(Fields::unit())
                    }),
            )
    }
}

/// A 'referencing view' for digest item. Does not own its contents. Used by
/// final runtime implementations for encoding/decoding its log items.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DigestItemRef<'a> {
    /// A pre-runtime digest.
    ///
    /// These are messages from the consensus engine to the runtime, although
    /// the consensus engine can (and should) read them itself to avoid
    /// code and state duplication.  It is erroneous for a runtime to produce
    /// these, but this is not (yet) checked.
    PreRuntime(&'a ConsensusEngineId, &'a [u8]),
    /// A message from the runtime to the consensus engine. This should *never*
    /// be generated by the native code of any consensus engine, but this is not
    /// checked (yet).
    Consensus(&'a ConsensusEngineId, &'a [u8]),
    /// Put a Seal on it. This is only used by native code, and is never seen
    /// by runtimes.
    Seal(&'a ConsensusEngineId, &'a [u8]),
    /// Any 'non-system' digest item, opaque to the native code.
    Other(&'a [u8]),
    /// Runtime code or heap pages updated.
    RuntimeEnvironmentUpdated,
}

/// Type of the digest item. Used to gain explicit control over `DigestItem` encoding
/// process. We need an explicit control, because final runtimes are encoding their own
/// digest items using `DigestItemRef` type and we can't auto-derive `Decode`
/// trait for `DigestItemRef`.
#[repr(u32)]
#[derive(Debug, Encode, Decode)]
pub enum DigestItemType {
    Other = 0u32,
    Consensus = 4u32,
    Seal = 5u32,
    PreRuntime = 6u32,
    RuntimeEnvironmentUpdated = 8u32,
}

/// Type of a digest item that contains raw data; this also names the consensus engine ID where
/// applicable. Used to identify one or more digest items of interest.
#[derive(Copy, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[allow(dead_code)]
pub enum OpaqueDigestItemId<'a> {
    /// Type corresponding to DigestItem::PreRuntime.
    PreRuntime(&'a ConsensusEngineId),
    /// Type corresponding to DigestItem::Consensus.
    Consensus(&'a ConsensusEngineId),
    /// Type corresponding to DigestItem::Seal.
    Seal(&'a ConsensusEngineId),
    /// Some other (non-prescribed) type.
    Other,
}

impl DigestItem {
    /// Returns a 'referencing view' for this digest item.
    pub fn dref(&self) -> DigestItemRef {
        match *self {
            Self::PreRuntime(ref v, ref s) => DigestItemRef::PreRuntime(v, s),
            Self::Consensus(ref v, ref s) => DigestItemRef::Consensus(v, s),
            Self::Seal(ref v, ref s) => DigestItemRef::Seal(v, s),
            Self::Other(ref v) => DigestItemRef::Other(v),
            Self::RuntimeEnvironmentUpdated => DigestItemRef::RuntimeEnvironmentUpdated,
        }
    }

    /// Returns `Some` if this entry is the `PreRuntime` entry.
    #[allow(dead_code)]
    pub fn as_pre_runtime(&self) -> Option<(ConsensusEngineId, &[u8])> {
        self.dref().as_pre_runtime()
    }

    /// Returns `Some` if this entry is the `Consensus` entry.
    #[allow(dead_code)]
    pub fn as_consensus(&self) -> Option<(ConsensusEngineId, &[u8])> {
        self.dref().as_consensus()
    }

    /// Returns `Some` if this entry is the `Seal` entry.
    #[allow(dead_code)]
    pub fn as_seal(&self) -> Option<(ConsensusEngineId, &[u8])> {
        self.dref().as_seal()
    }

    /// Returns Some if `self` is a `DigestItem::Other`.
    #[allow(dead_code)]
    pub fn as_other(&self) -> Option<&[u8]> {
        self.dref().as_other()
    }

    /// Returns the opaque data contained in the item if `Some` if this entry has the id given.
    #[allow(dead_code)]
    pub fn try_as_raw(&self, id: OpaqueDigestItemId) -> Option<&[u8]> {
        self.dref().try_as_raw(id)
    }

    /// Returns the data contained in the item if `Some` if this entry has the id given, decoded
    /// to the type provided `T`.
    #[allow(dead_code)]
    pub fn try_to<T: Decode>(&self, id: OpaqueDigestItemId) -> Option<T> {
        self.dref().try_to::<T>(id)
    }

    /// Try to match this to a `Self::Seal`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a seal item, the `id` doesn't match or when the decoding fails.
    #[allow(dead_code)]
    pub fn seal_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        self.dref().seal_try_to(id)
    }

    /// Try to match this to a `Self::Consensus`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a consensus item, the `id` doesn't match or
    /// when the decoding fails.
    #[allow(dead_code)]
    pub fn consensus_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        self.dref().consensus_try_to(id)
    }

    /// Try to match this to a `Self::PreRuntime`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a pre-runtime item, the `id` doesn't match or
    /// when the decoding fails.
    #[allow(dead_code)]
    pub fn pre_runtime_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        self.dref().pre_runtime_try_to(id)
    }
}

impl Encode for DigestItem {
    fn encode(&self) -> Vec<u8> {
        self.dref().encode()
    }
}

impl scale::EncodeLike for DigestItem {}

impl Decode for DigestItem {
    #[allow(deprecated)]
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let item_type: DigestItemType = Decode::decode(input)?;
        match item_type {
            DigestItemType::PreRuntime => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::PreRuntime(vals.0, vals.1))
            }
            DigestItemType::Consensus => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Consensus(vals.0, vals.1))
            }
            DigestItemType::Seal => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Seal(vals.0, vals.1))
            }
            DigestItemType::Other => Ok(Self::Other(Decode::decode(input)?)),
            DigestItemType::RuntimeEnvironmentUpdated => Ok(Self::RuntimeEnvironmentUpdated),
        }
    }
}

impl<'a> DigestItemRef<'a> {
    /// Cast this digest item into `PreRuntime`
    pub fn as_pre_runtime(&self) -> Option<(ConsensusEngineId, &'a [u8])> {
        match *self {
            Self::PreRuntime(consensus_engine_id, data) => Some((*consensus_engine_id, data)),
            _ => None,
        }
    }

    /// Cast this digest item into `Consensus`
    pub fn as_consensus(&self) -> Option<(ConsensusEngineId, &'a [u8])> {
        match *self {
            Self::Consensus(consensus_engine_id, data) => Some((*consensus_engine_id, data)),
            _ => None,
        }
    }

    /// Cast this digest item into `Seal`
    pub fn as_seal(&self) -> Option<(ConsensusEngineId, &'a [u8])> {
        match *self {
            Self::Seal(consensus_engine_id, data) => Some((*consensus_engine_id, data)),
            _ => None,
        }
    }

    /// Cast this digest item into `PreRuntime`
    pub fn as_other(&self) -> Option<&'a [u8]> {
        match *self {
            Self::Other(data) => Some(data),
            _ => None,
        }
    }

    /// Try to match this digest item to the given opaque item identifier; if it matches, then
    /// return the opaque data it contains.
    pub fn try_as_raw(&self, id: OpaqueDigestItemId) -> Option<&'a [u8]> {
        match (id, self) {
            (OpaqueDigestItemId::Consensus(w), &Self::Consensus(v, s))
            | (OpaqueDigestItemId::Seal(w), &Self::Seal(v, s))
            | (OpaqueDigestItemId::PreRuntime(w), &Self::PreRuntime(v, s))
                if v == w =>
            {
                Some(s)
            }
            (OpaqueDigestItemId::Other, &Self::Other(s)) => Some(s),
            _ => None,
        }
    }

    /// Try to match this digest item to the given opaque item identifier; if it matches, then
    /// try to cast to the given data type; if that works, return it.
    pub fn try_to<T: Decode>(&self, id: OpaqueDigestItemId) -> Option<T> {
        self.try_as_raw(id)
            .and_then(|mut x| Decode::decode(&mut x).ok())
    }

    /// Try to match this to a `Self::Seal`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a seal item, the `id` doesn't match or when the decoding fails.
    pub fn seal_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        match self {
            Self::Seal(v, s) if *v == id => Decode::decode(&mut &s[..]).ok(),
            _ => None,
        }
    }

    /// Try to match this to a `Self::Consensus`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a consensus item, the `id` doesn't match or
    /// when the decoding fails.
    pub fn consensus_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        match self {
            Self::Consensus(v, s) if *v == id => Decode::decode(&mut &s[..]).ok(),
            _ => None,
        }
    }

    /// Try to match this to a `Self::PreRuntime`, check `id` matches and decode it.
    ///
    /// Returns `None` if this isn't a pre-runtime item, the `id` doesn't match or
    /// when the decoding fails.
    pub fn pre_runtime_try_to<T: Decode>(&self, id: &ConsensusEngineId) -> Option<T> {
        match self {
            Self::PreRuntime(v, s) if *v == id => Decode::decode(&mut &s[..]).ok(),
            _ => None,
        }
    }
}

impl<'a> Encode for DigestItemRef<'a> {
    fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();

        match *self {
            Self::Consensus(val, data) => {
                DigestItemType::Consensus.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Seal(val, sig) => {
                DigestItemType::Seal.encode_to(&mut v);
                (val, sig).encode_to(&mut v);
            }
            Self::PreRuntime(val, data) => {
                DigestItemType::PreRuntime.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Other(val) => {
                DigestItemType::Other.encode_to(&mut v);
                val.encode_to(&mut v);
            }
            Self::RuntimeEnvironmentUpdated => {
                DigestItemType::RuntimeEnvironmentUpdated.encode_to(&mut v);
            }
        }

        v
    }
}

impl<'a> scale::EncodeLike for DigestItemRef<'a> {}
