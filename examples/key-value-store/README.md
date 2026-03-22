# Key-Value Store

An in-memory key-value store using IRIS State (BTreeMap) operations.

## How it works

The store is a State value (map) that supports standard CRUD operations:
- `kv_set` / `kv_get` - store and retrieve values by string key
- `kv_delete` - remove a key
- `kv_has` - check key existence
- `kv_keys` / `kv_values` - enumerate contents
- `kv_size` - count entries
- `kv_merge` - combine two stores

State is immutable: each mutation returns a new state value.

## Running

```bash
iris run examples/key-value-store/kv-store.iris
iris run examples/key-value-store/kv-test.iris
```

## Primitives used

- `state_empty` - create empty map
- `map_insert` / `map_get` / `map_remove` - map CRUD
- `map_keys` / `map_values` / `map_size` - map queries
- `fold` - iteration over key lists
