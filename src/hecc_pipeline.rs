use crate::hecc_core::{hecc_decode, hecc_encode, HeccParams};
use rand::RngCore;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeccShard {
    pub block: u32,
    pub index: u8,
    pub value: u8,
}

#[derive(Debug)]
pub enum HeccPipelineError {
    InvalidParams,
    NotEnoughShards { block: u32, have: usize, need: usize },
    InvalidHeader,
    LengthOverflow,
}

/// High-level API: shred arbitrary bytes into HECC shards with block metadata.
pub fn hecc_shred_bytes(
    params: HeccParams,
    message: &[u8],
    rng: &mut impl RngCore,
) -> Result<Vec<HeccShard>, HeccPipelineError> {
    params.validate().map_err(|_| HeccPipelineError::InvalidParams)?;
    let k = params.k;
    let t = params.t;
    let n = params.n;

    if message.len() > u32::MAX as usize {
        return Err(HeccPipelineError::LengthOverflow);
    }

    let mut payload = Vec::with_capacity(4 + message.len());
    payload.extend_from_slice(&(message.len() as u32).to_be_bytes());
    payload.extend_from_slice(message);

    let mut shards = Vec::new();
    let mut block: u32 = 0;
    let mut offset = 0;
    while offset < payload.len() {
        let end = usize::min(offset + k, payload.len());
        let mut m = vec![0u8; k];
        m[..end - offset].copy_from_slice(&payload[offset..end]);

        let mut r = vec![0u8; t];
        rng.fill_bytes(&mut r);

        let encoded = hecc_encode(&m, &r, n).map_err(|_| HeccPipelineError::InvalidParams)?;
        for (i, &value) in encoded.iter().enumerate() {
            shards.push(HeccShard {
                block,
                index: (i + 1) as u8,
                value,
            });
        }

        block = block.wrapping_add(1);
        offset = end;
    }

    Ok(shards)
}

/// High-level API: recover original bytes from HECC shards.
pub fn hecc_recover_bytes(
    params: HeccParams,
    shards: &[HeccShard],
) -> Result<Vec<u8>, HeccPipelineError> {
    params.validate().map_err(|_| HeccPipelineError::InvalidParams)?;
    let k = params.k;
    let t = params.t;
    let need = k + t;

    let mut by_block: BTreeMap<u32, Vec<HeccShard>> = BTreeMap::new();
    for shard in shards {
        by_block.entry(shard.block).or_default().push(*shard);
    }

    let mut recovered = Vec::new();
    for (block, mut list) in by_block {
        if list.len() < need {
            return Err(HeccPipelineError::NotEnoughShards {
                block,
                have: list.len(),
                need,
            });
        }

        let mut seen = [false; 256];
        let mut pairs = Vec::with_capacity(need);
        for shard in list.drain(..) {
            let idx = shard.index as usize;
            if idx == 0 || idx > 255 || seen[idx] {
                continue;
            }
            seen[idx] = true;
            pairs.push((shard.value, idx));
            if pairs.len() == need {
                break;
            }
        }

        if pairs.len() < need {
            return Err(HeccPipelineError::NotEnoughShards {
                block,
                have: pairs.len(),
                need,
            });
        }

        let (m, _) = hecc_decode(&pairs, k, t).map_err(|_| HeccPipelineError::InvalidParams)?;
        recovered.extend_from_slice(&m);
    }

    if recovered.len() < 4 {
        return Err(HeccPipelineError::InvalidHeader);
    }
    let len = u32::from_be_bytes([recovered[0], recovered[1], recovered[2], recovered[3]]) as usize;
    if recovered.len() < 4 + len {
        return Err(HeccPipelineError::InvalidHeader);
    }
    Ok(recovered[4..4 + len].to_vec())
}
