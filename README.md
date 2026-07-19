# opencollection

A typed Rust data model and YAML (de)serializer for the
[OpenCollection](https://spec.opencollection.com) API-collection format
(v1.0.0) — the open, YAML-based collection spec created by
[Bruno](https://docs.usebruno.com/opencollection-yaml/overview).

The full JSON Schema is modeled with serde: HTTP, GraphQL, gRPC and WebSocket
requests, folders, script files, environments (with secret variables and value
variants), every auth scheme (Basic, Bearer, Digest, NTLM, WSSE, API key,
AWS SigV4, OAuth 1.0 and all four OAuth 2.0 flows, plus `inherit`), all body
kinds, assertions, runtime actions, proxy/protobuf/client-certificate config,
and `value | "inherit"` settings.

- **Lossless**: parse → modify → serialize round-trips are value-identical.
- **Strict**: unknown fields are rejected wherever the schema forbids them.
- **Validated**: the test suite checks serialized output against the official
  schema at `schema.opencollection.com`.

## Usage

```rust
use opencollection::{Auth, Environment, Folder, HttpRequest, OpenCollection};

// Parse an existing collection — a single file or a directory tree
let collection = OpenCollection::load("opencollection.yml")?;
for item in collection.requests() {
    println!("{}", item.name().unwrap_or("<unnamed>"));
}

// Or build one fluently
let collection = OpenCollection::new("Petstore")
    .environment(
        Environment::new("prod")
            .variable("baseUrl", "https://api.example.com")
            .secret("apiKey"),
    )
    .item(
        Folder::new("Pets").item(
            HttpRequest::get("{{baseUrl}}/pets")
                .name("List pets")
                .query("limit", "10")
                .auth(Auth::bearer("{{apiKey}}")),
        ),
    );
collection.save("opencollection.yml")?;
```

All fields are public, so the builder methods can be freely mixed with direct
struct access.

## Bundled and unbundled collections

The spec stores a collection either as one standalone file or as a tree of
folders and files, and the `bundled` flag says which. `save` follows it: with
`bundled: false` the path is a directory and the collection is exploded into a
tree, otherwise it is a single YAML file.

```rust
collection.bundled = Some(false);
collection.save("petstore")?;
```

```text
petstore/
├── opencollection.yml   # everything but the items
├── list-pets.yml
└── pet-by-owner/
    ├── folder.yml
    └── create-pet.yml
```

`load` reads either layout, taking a directory as a tree and anything else as a
single file, and rejects a collection whose `bundled` flag disagrees with how it
is actually stored.

### File names are identity, not a projection of the title

An item loaded from a tree remembers the file it came from and is written back
to it. Renaming a request therefore does **not** rename its file — the same
model Bruno uses, where renaming the title and renaming the file are separate
operations.

Only items that have never been saved get a name derived from their title, using
the same rules as Bruno's `sanitizeName`: replace the characters a filesystem
rejects, keep everything else, so `"Create user"` becomes `Create user.yml`
rather than `create-user.yml`. Names are optional throughout the schema — `info`
is optional on every request, and a script file has no name at all — so nameless
items fall back to their type (`http.yml`, `script.yml`). Collisions get a `-2`,
`-3` suffix, compared case-insensitively because `Get.yml` and `get.yml` are one
file on macOS and Windows.

This is what makes load → save → load idempotent. Deriving names afresh on every
save renames every file in a tree written by another tool, and since the old
files are still there, every item comes back twice.

### Saving prunes, but only where it has standing

Saving back into the directory a collection was loaded from deletes the item
files whose items are gone, so moving and deleting items works. It only ever
removes things it would have read as items — a `.yml`/`.yaml` file, or a
directory holding a `folder.yml`. The `.proto`, `.env`, `.pem` and README files
that live beside requests are never touched, and neither is anything in a
directory the collection was not loaded from.

### Reloading one item

```rust
match collection.load_item("pets/create-pet.yml") {
    Ok(item)                    => { /* replace it in the tree, or add it */ }
    Err(Error::Deleted { .. })  => { /* the item was removed on disk */ }
    Err(Error::NotAnItem { .. }) => { /* a README, a .proto, a config file */ }
    Err(error)                  => return Err(error),
}
```

`load_item` parses one file and hands it back with its `Source` set. It does
**not** splice it into the collection: whether an incoming version may replace an
item the user is editing is your policy, not the library's. Match it against the
tree by `Item::source` to replace it, or push it to add it.

The path may be absolute or relative to the collection root, and may name either
a folder's directory or its `folder.yml` — both give you the folder with its
subtree. Since a watcher reports every path in the tree, `NotAnItem` is a normal
event to ignore rather than a failure.

There is deliberately **no file watcher here**. Delivering events would force a
channel or an async runtime on you, and suppressing the events your own `save`
produces requires knowing your save lifecycle. Bring your own watcher and call
`load_item`.

### Saving one item

```rust
collection.save_item(item, "petstore")?;
```

In a tree this writes just that item's file — for a folder, its `folder.yml` and
everything beneath it — plus the root `opencollection.yml` if it actually
changed. In a bundled collection the item's home is the whole document, so this
is exactly `save`, and every other pending change lands with it.

### Two things a tree does not record

- **Item order.** Items are read back in directory order. Set `seq` on the ones
  whose order matters.
- **Comments.** Serialization goes through serde, which does not preserve them,
  so a hand-written comment is lost the first time that file is saved. Saving a
  single item limits this to the file you touched.

## License

MIT
