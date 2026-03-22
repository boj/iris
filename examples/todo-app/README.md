# TODO App

A TODO list manager using IRIS State for persistent storage.

## How it works

Each todo item is stored in a State map with the key `item_N` where N is an
auto-incrementing ID. Items are tuples of `(description, status)` where
status is 0 for pending and 1 for done.

Operations:
- `todo_add` - add a new item (returns updated store and item ID)
- `todo_complete` - mark an item as done
- `todo_remove` - delete an item
- `todo_get` - retrieve an item's description and status
- `todo_count_pending` / `todo_count_done` - count by status
- `todo_list_keys` - list all item keys

## Running

```bash
iris run examples/todo-app/todo.iris
iris run examples/todo-app/todo-test.iris
```

## Primitives used

- `state_empty` / `map_insert` / `map_get` / `map_remove` - map operations
- `map_keys` - enumerate keys
- `str_starts_with` - filter item keys from metadata keys
- `str_concat` / `int_to_string` - key generation
- `fold` / `filter` - iteration and counting
