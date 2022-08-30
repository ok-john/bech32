## bech32

A tiny rust implementation the Bech32 encoding standard ([BIP173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)).

## Usage

### Encoding
```rust
let result: Vec<u8> = bech32::encode(hrp, data);
```

### Decoding
```rust
let (hrp: Vec<u8>, data: Vec<u8>) = bech32::decode(hrp, data);
```
