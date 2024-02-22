## 0.9.0 (TBD)

## 0.8.1 (2024-02-21)

* Fixed clippy warnings (#280)

## 0.8.0 (2024-02-14)

* Implemented the `PartialMmr` data structure (#195).
* Implemented RPX hash function (#201).
* Added `FeltRng` and `RpoRandomCoin` (#237).
* Accelerated RPO/RPX hash functions using AVX512 instructions (#234).
* Added `inner_nodes()` method to `PartialMmr` (#238).
* Improved `PartialMmr::apply_delta()` (#242).
* Refactored `SimpleSmt` struct (#245).
* Replaced `TieredSmt` struct with `Smt` struct (#254, #277).
* Updated Winterfell dependency to v0.8 (#275).

## 0.7.1 (2023-10-10)

* Fixed RPO Falcon signature build on Windows.

## 0.7.0 (2023-10-05)

* Replaced `MerklePathSet` with `PartialMerkleTree` (#165).
* Implemented clearing of nodes in `TieredSmt` (#173).
* Added ability to generate inclusion proofs for `TieredSmt` (#174).
* Implemented Falcon DSA (#179).
* Added conditional `serde`` support for various structs (#180).
* Implemented benchmarking for `TieredSmt` (#182).
* Added more leaf traversal methods for `MerkleStore` (#185).
* Added SVE acceleration for RPO hash function (#189).

## 0.6.0 (2023-06-25)

* [BREAKING] Added support for recording capabilities for `MerkleStore` (#162).
* [BREAKING] Refactored Merkle struct APIs to use `RpoDigest` instead of `Word` (#157).
* Added initial implementation of `PartialMerkleTree` (#156).

## 0.5.0 (2023-05-26)

* Implemented `TieredSmt` (#152, #153).
* Implemented ability to extract a subset of a `MerkleStore` (#151).
* Cleaned up `SimpleSmt` interface (#149).
* Decoupled hashing and padding of peaks in `Mmr` (#148).
* Added `inner_nodes()` to `MerkleStore` (#146).

## 0.4.0 (2023-04-21)

- Exported `MmrProof` from the crate (#137).
- Allowed merging of leaves in `MerkleStore` (#138).
- [BREAKING] Refactored how existing data structures are added to `MerkleStore` (#139).

## 0.3.0 (2023-04-08)

- Added `depth` parameter to SMT constructors in `MerkleStore` (#115).
- Optimized MMR peak hashing for Miden VM (#120).
- Added `get_leaf_depth` method to `MerkleStore` (#119).
- Added inner node iterators to `MerkleTree`, `SimpleSmt`, and `Mmr` (#117, #118, #121).

## 0.2.0 (2023-03-24)

- Implemented `Mmr` and related structs (#67).
- Implemented `MerkleStore` (#93, #94, #95, #107 #112).
- Added benchmarks for `MerkleStore` vs. other structs (#97).
- Added Merkle path containers (#99).
- Fixed depth handling in `MerklePathSet` (#110).
- Updated Winterfell dependency to v0.6.

## 0.1.4 (2023-02-22)

- Re-export winter-crypto Hasher, Digest & ElementHasher (#72)

## 0.1.3 (2023-02-20)

- Updated Winterfell dependency to v0.5.1 (#68)

## 0.1.2 (2023-02-17)

- Fixed `Rpo256::hash` pad that was panicking on input (#44)
- Added `MerklePath` wrapper to encapsulate Merkle opening verification and root computation (#53)
- Added `NodeIndex` Merkle wrapper to encapsulate Merkle tree traversal and mappings (#54)

## 0.1.1 (2023-02-06)

- Introduced `merge_in_domain` for the RPO hash function, to allow using a specified domain value in the second capacity register when hashing two digests together.
- Added a simple sparse Merkle tree implementation.
- Added re-exports of Winterfell RandomCoin and RandomCoinError.

## 0.1.0 (2022-12-02)

- Initial release on crates.io containing the cryptographic primitives used in Miden VM and the Miden Rollup.
- Hash module with the BLAKE3 and Rescue Prime Optimized hash functions.
    - BLAKE3 is implemented with 256-bit, 192-bit, or 160-bit output.
    - RPO is implemented with 256-bit output.
- Merkle module, with a set of data structures related to Merkle trees, implemented using the RPO hash function.
