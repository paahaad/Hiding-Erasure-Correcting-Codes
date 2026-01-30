use hecc::{
    hecc_decode, hecc_encode, hecc_recover_bytes, hecc_shred_bytes, HeccParams,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand::seq::SliceRandom;

#[test]
fn hecc_roundtrip_small() {
    let m = vec![1u8, 2, 3];
    let r = vec![9u8, 8];
    let shreds = hecc_encode(&m, &r, 8).expect("encode");
    let shards = vec![
        (shreds[0], 1),
        (shreds[2], 3),
        (shreds[4], 5),
        (shreds[5], 6),
        (shreds[7], 8),
    ];
    let (m2, r2) = hecc_decode(&shards, m.len(), r.len()).expect("decode");
    assert_eq!(m2, m);
    assert_eq!(r2, r);
}

#[test]
fn hecc_roundtrip_random() {
    let mut rng = StdRng::seed_from_u64(123);
    let k = 10;
    let t = 4;
    let n = 20;
    for _ in 0..25 {
        let m: Vec<u8> = (0..k).map(|_| rng.gen()).collect();
        let r: Vec<u8> = (0..t).map(|_| rng.gen()).collect();
        let shreds = hecc_encode(&m, &r, n).expect("encode");

        let mut shards = Vec::new();
        let mut idxs: Vec<usize> = (1..=n).collect();
        idxs.shuffle(&mut rng);
        for idx in idxs.into_iter().take(k + t) {
            shards.push((shreds[idx - 1], idx));
        }

        let (m2, r2) = hecc_decode(&shards, k, t).expect("decode");
        assert_eq!(m2, m);
        assert_eq!(r2, r);
    }
}

#[test]
fn hecc_not_enough_shards() {
    let m = vec![1u8, 2, 3, 4];
    let r = vec![5u8, 6];
    let shreds = hecc_encode(&m, &r, 10).expect("encode");
    let shards = vec![(shreds[0], 1), (shreds[1], 2), (shreds[2], 3)];
    let err = hecc_decode(&shards, m.len(), r.len()).err();
    assert!(err.is_some());
}

#[test]
fn hecc_pipeline_roundtrip() {
    let params = HeccParams { k: 16, t: 6, n: 32 };
    let mut rng = StdRng::seed_from_u64(999);
    let message: Vec<u8> = (0..200).map(|_| rng.gen()).collect();

    let shards = hecc_shred_bytes(params, &message, &mut rng).expect("shred");

    // Keep only K+T shards per block (simulate erasures).
    let mut by_block = std::collections::BTreeMap::<u32, Vec<_>>::new();
    for shard in shards {
        by_block.entry(shard.block).or_default().push(shard);
    }
    let mut kept = Vec::new();
    for (_block, mut list) in by_block {
        list.shuffle(&mut rng);
        kept.extend(list.into_iter().take(params.k + params.t));
    }

    let recovered = hecc_recover_bytes(params, &kept).expect("recover");
    assert_eq!(recovered, message);
}

#[test]
fn hecc_invalid_params_rejected() {
    let m = vec![1u8, 2, 3];
    let r = vec![4u8];
    assert!(hecc_encode(&m, &r, 3).is_err());
    assert!(hecc_encode(&m, &r, 2).is_err());
    assert!(hecc_encode(&[], &r, 5).is_err());
}

#[test]
fn hecc_duplicate_indices_rejected() {
    let m = vec![1u8, 2, 3];
    let r = vec![9u8, 8];
    let shreds = hecc_encode(&m, &r, 8).expect("encode");
    let shards = vec![
        (shreds[0], 1),
        (shreds[1], 2),
        (shreds[2], 3),
        (shreds[2], 3),
        (shreds[3], 4),
    ];
    assert!(hecc_decode(&shards, m.len(), r.len()).is_err());
}

#[test]
fn hecc_invalid_index_rejected() {
    let m = vec![1u8, 2, 3];
    let r = vec![9u8, 8];
    let shreds = hecc_encode(&m, &r, 8).expect("encode");
    let shards = vec![
        (shreds[0], 0),
        (shreds[1], 2),
        (shreds[2], 3),
        (shreds[3], 4),
        (shreds[4], 5),
    ];
    assert!(hecc_decode(&shards, m.len(), r.len()).is_err());
}

#[test]
fn hecc_pipeline_missing_block_fails() {
    let params = HeccParams { k: 8, t: 4, n: 16 };
    let mut rng = StdRng::seed_from_u64(2024);
    let message: Vec<u8> = (0..120).map(|_| rng.gen()).collect();
    let shards = hecc_shred_bytes(params, &message, &mut rng).expect("shred");

    let mut by_block = std::collections::BTreeMap::<u32, Vec<_>>::new();
    for shard in shards {
        by_block.entry(shard.block).or_default().push(shard);
    }

    let mut kept = Vec::new();
    let mut first = true;
    for (_block, mut list) in by_block {
        list.shuffle(&mut rng);
        let take = if first { params.k + params.t - 1 } else { params.k + params.t };
        kept.extend(list.into_iter().take(take));
        first = false;
    }

    assert!(hecc_recover_bytes(params, &kept).is_err());
}
