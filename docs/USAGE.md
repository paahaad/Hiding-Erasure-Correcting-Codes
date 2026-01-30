# Usage Notes

This document explains the core API concepts and how to integrate HECC into your application.

## Parameters

`HeccParams` controls the encoding scheme:

- `k`: number of data shards
- `t`: number of redundancy shards
- `n`: total shards, where `n = k + t`

All pipeline helpers operate on these parameters.

## Core API

Use `hecc_encode` and `hecc_decode` when you already have the message bytes and redundancy bytes.

```rust
use hecc::{hecc_encode, hecc_decode};

let m = vec![1u8, 2, 3];
let r = vec![9u8, 8];
let shreds = hecc_encode(&m, &r, 7).unwrap();
let shards = vec![
    (shreds[0], 1),
    (shreds[2], 3),
    (shreds[4], 5),
    (shreds[5], 6),
    (shreds[6], 7),
];
let (m2, r2) = hecc_decode(&shards, m.len(), r.len()).unwrap();
assert_eq!(m2, m);
assert_eq!(r2, r);
```

## Pipeline API

Use `hecc_shred_bytes` and `hecc_recover_bytes` when working with raw byte payloads.

```rust
use hecc::{hecc_recover_bytes, hecc_shred_bytes, HeccParams};
use rand::{rngs::StdRng, SeedableRng};

let params = HeccParams { k: 8, t: 4, n: 16 };
let mut rng = StdRng::seed_from_u64(1);
let payload = b"hello pipeline";
let shards = hecc_shred_bytes(params, payload, &mut rng).unwrap();
let recovered = hecc_recover_bytes(params, &shards).unwrap();
assert_eq!(recovered, payload);
```

## Error Handling

- `HeccError` and `HeccPipelineError` enumerate expected failure modes.
- Validate shard indices and lengths before calling decode or recovery APIs.
