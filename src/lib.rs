//! A typed data model and YAML (de)serializer for the
//! [OpenCollection](https://spec.opencollection.com) API-collection format,
//! v1.0.0.
//!
//! OpenCollection is an open, YAML-based specification for API collections
//! (HTTP, GraphQL, gRPC and WebSocket requests, folders, environments,
//! scripts, …). This crate models the full schema with serde, so collections
//! can be parsed, inspected, modified and written back losslessly.
//!
//! # Example
//!
//! ```
//! use opencollection::{Auth, Environment, Folder, HttpRequest, OpenCollection};
//!
//! let collection = OpenCollection::new("Petstore")
//!     .summary("Example collection")
//!     .environment(
//!         Environment::new("prod")
//!             .variable("baseUrl", "https://api.example.com")
//!             .secret("apiKey"),
//!     )
//!     .item(
//!         Folder::new("Pets")
//!             .item(
//!                 HttpRequest::get("{{baseUrl}}/pets")
//!                     .name("List pets")
//!                     .query("limit", "10")
//!                     .auth(Auth::bearer("{{apiKey}}")),
//!             )
//!             .item(
//!                 HttpRequest::post("{{baseUrl}}/pets")
//!                     .name("Create pet")
//!                     .header("Content-Type", "application/json")
//!                     .json_body(r#"{"name": "Rex"}"#),
//!             ),
//!     );
//!
//! // Collections are read and written through the filesystem, so that the
//! // `bundled` flag always decides the layout — see `save`.
//! let path = std::env::temp_dir().join("opencollection-doc-example.yml");
//! collection.save(&path).unwrap();
//!
//! let parsed = OpenCollection::load(&path).unwrap();
//! assert_eq!(collection, parsed);
//! assert_eq!(parsed.iter().filter(|item| item.is_request()).count(), 2);
//! # std::fs::remove_file(&path).unwrap();
//! ```

mod auth;
mod body;
mod common;
mod config;
mod item;
mod request;

pub use auth::*;
pub use body::*;
pub use common::*;
pub use config::*;
pub use item::*;
pub use request::*;

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Free-form YAML value and mapping, re-exported for the
/// [`OpenCollection::extensions`] field.
pub use serde_yaml_ng::{Mapping, Value};

/// An OpenCollection document (the root object of an `opencollection.yml`).
///
/// Unlike the nested types, the root object accepts unknown fields, matching
/// the schema.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OpenCollection {
    /// Info about the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Info>,
    /// The version of the OpenCollection spec this document targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencollection: Option<String>,
    /// Configuration for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<CollectionConfig>,
    /// The items in the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
    /// Default request configuration for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestDefaults>,
    /// Documentation for the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Documentation>,
    /// `true` if the collection is a standalone file, `false` if stored on the
    /// filesystem as a nested structure of folders and files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundled: Option<bool>,
    /// Free-form extension data.
    ///
    /// A [`Mapping`] rather than a `BTreeMap` so that key order survives a
    /// round-trip; this is the one place implementers put arbitrary data, so
    /// reordering it would be a visible, unexplained diff in their files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Mapping>,
    /// The directory this collection was loaded from, for unbundled
    /// collections; see [`Source`].
    ///
    /// It anchors the relative paths its items carry, and gates pruning: saving
    /// back into this directory removes files whose items are gone, saving
    /// anywhere else never deletes.
    #[serde(skip)]
    pub source: Source,
}

impl OpenCollection {
    /// Create a collection with the given name, targeting spec version `1.0.0`.
    ///
    /// This and the other fluent helpers are plain methods, and every field
    /// stays `pub`, so builder calls and direct field access mix freely.
    ///
    /// ```
    /// use opencollection::OpenCollection;
    ///
    /// let mut collection = OpenCollection::new("Petstore");
    /// collection.bundled = Some(false); // no builder needed
    ///
    /// assert_eq!(collection.opencollection.as_deref(), Some("1.0.0"));
    /// assert_eq!(
    ///     collection.info.as_ref().unwrap().name.as_deref(),
    ///     Some("Petstore"),
    /// );
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        OpenCollection {
            opencollection: Some("1.0.0".to_owned()),
            info: Some(Info {
                name: Some(name.into()),
                ..Info::default()
            }),
            ..OpenCollection::default()
        }
    }

    /// Set the collection summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.info.get_or_insert_with(Info::default).summary = Some(summary.into());
        self
    }

    /// Set the collection version (the collection's own version, not the spec's).
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.info.get_or_insert_with(Info::default).version = Some(version.into());
        self
    }

    /// Append an item: a request of any protocol, a folder, or a script file.
    ///
    /// ```
    /// use opencollection::{Folder, GrpcRequest, HttpRequest, OpenCollection};
    ///
    /// let collection = OpenCollection::new("Petstore")
    ///     .item(HttpRequest::get("https://example.com/pets").name("List pets"))
    ///     .item(GrpcRequest::new("grpc.example.com", "pets.Pets/Watch").name("Watch pets"))
    ///     .item(
    ///         Folder::new("Admin")
    ///             .item(HttpRequest::post("https://example.com/pets").name("Create pet")),
    ///     );
    ///
    /// // Folders are items too, and `iter` descends into them.
    /// assert_eq!(collection.iter().count(), 4);
    /// assert_eq!(collection.requests().count(), 3);
    /// ```
    pub fn item(mut self, item: impl Into<Item>) -> Self {
        self.items.get_or_insert_with(Vec::new).push(item.into());
        self
    }

    /// Append an environment to the collection config.
    ///
    /// ```
    /// use opencollection::{Environment, OpenCollection};
    ///
    /// let collection = OpenCollection::new("Petstore")
    ///     .environment(
    ///         Environment::new("local").variable("baseUrl", "http://localhost:3000"),
    ///     )
    ///     .environment(
    ///         Environment::new("prod")
    ///             .variable("baseUrl", "https://api.example.com")
    ///             // Declared but never given a value in the file.
    ///             .secret("apiKey"),
    ///     );
    ///
    /// let environments = collection.config.unwrap().environments.unwrap();
    /// assert_eq!(environments.len(), 2);
    /// ```
    pub fn environment(mut self, environment: Environment) -> Self {
        self.config
            .get_or_insert_with(CollectionConfig::default)
            .environments
            .get_or_insert_with(Vec::new)
            .push(environment);
        self
    }

    /// Read a collection from the filesystem, in either layout.
    ///
    /// A **directory** is read as an unbundled collection: `opencollection.yml`
    /// at its root supplies everything except the items, which come from the
    /// tree around it. **Any other path** is read as a single bundled file.
    ///
    /// ```
    /// use opencollection::{HttpRequest, OpenCollection};
    /// # let dir = std::env::temp_dir().join("opencollection-doc-load");
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # let mut seed = OpenCollection::new("Petstore")
    /// #     .item(HttpRequest::get("https://example.com/pets").name("List pets"));
    /// # seed.bundled = Some(false);
    /// # seed.save(&dir)?;
    ///
    /// let collection = OpenCollection::load(&dir)?;
    /// for request in collection.requests() {
    ///     println!("{}", request.name().unwrap_or("<unnamed>"));
    /// }
    /// assert_eq!(collection.requests().count(), 1);
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # Ok::<(), opencollection::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Both layouts are checked against the [`bundled`](Self::bundled) flag they
    /// declare, so a collection stored one way but claiming the other is
    /// reported rather than half-read — reading the `opencollection.yml` of a
    /// tree on its own would otherwise succeed and hand back zero items, an
    /// empty result that looks like success.
    ///
    /// - [`Error::Layout`] for a directory with no `opencollection.yml`, a tree
    ///   declaring `bundled: true`, or a lone file declaring `bundled: false`.
    /// - [`Error::Parse`] naming the offending file, which in a tree of fifty
    ///   files is the difference between a usable error and a riddle.
    /// - [`Error::Io`] if a file cannot be read.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if path.is_dir() {
            return read_unbundled(path);
        }

        let yaml = std::fs::read_to_string(path).map_err(|error| Error::io(path, error))?;
        let collection: Self = serde_yaml_ng::from_str(&yaml).map_err(|source| Error::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        // A single file declaring `bundled: false` is the root of a tree, and
        // reading it alone yields a collection with no items — an empty result
        // that looks like success. Point at the directory instead.
        if collection.bundled == Some(false) {
            let directory = path.parent().unwrap_or(Path::new("."));
            return Err(Error::layout(
                path,
                format!(
                    "declares `bundled: false`, so it is the root of an unbundled \
                     collection: read the directory `{}` instead",
                    directory.display()
                ),
            ));
        }

        Ok(collection)
    }

    /// Write the collection to `path`, in the layout its
    /// [`bundled`](Self::bundled) flag calls for.
    ///
    /// - `Some(false)` — `path` is a **directory**, created if needed, and the
    ///   collection is exploded into a tree: the root document goes to
    ///   `path/opencollection.yml`, each request and script becomes its own
    ///   `.yml`, and each folder becomes a subdirectory holding a `folder.yml`
    ///   beside its children.
    /// - anything else — `path` is a **single YAML file** holding everything.
    ///
    /// ```
    /// use opencollection::{Folder, HttpRequest, OpenCollection};
    /// # let file = std::env::temp_dir().join("opencollection-doc-save.yml");
    /// # let dir = std::env::temp_dir().join("opencollection-doc-save-tree");
    /// # let _ = std::fs::remove_dir_all(&dir);
    ///
    /// let mut collection = OpenCollection::new("Petstore").item(
    ///     Folder::new("Pets")
    ///         .item(HttpRequest::get("https://example.com/pets").name("List pets")),
    /// );
    ///
    /// collection.save(&file)?; // one document
    ///
    /// collection.bundled = Some(false);
    /// collection.save(&dir)?; // a tree
    /// assert!(dir.join("Pets/List pets.yml").is_file());
    /// # let _ = std::fs::remove_file(&file);
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # Ok::<(), opencollection::Error>(())
    /// ```
    ///
    /// # File names in a tree
    ///
    /// An item loaded from disk is written back to the file it came from, so
    /// **renaming a request does not rename its file** — the two are separate
    /// operations, as they are in Bruno. Only an item that has never been saved
    /// gets a name derived from its title, and that keeps everything a
    /// filesystem allows: `"Create user"` becomes `Create user.yml`.
    ///
    /// Nameless items — `info` is optional on every request and [`ScriptFile`]
    /// has no name at all — fall back to their type (`http.yml`, `script.yml`).
    /// Collisions take a `-2`, `-3` suffix, compared case-insensitively because
    /// `Get.yml` and `get.yml` are one file on macOS and Windows.
    ///
    /// # What gets deleted
    ///
    /// Saving back into the directory this collection was [`load`](Self::load)ed
    /// from deletes the item files whose items are gone, which is what makes
    /// moving and deleting items work. Only files that would be read back as
    /// items are ever removed: the `.proto`, `.env`, `.pem` and README files
    /// living beside the requests are untouched, and so is everything in a
    /// directory this collection was not loaded from.
    ///
    /// # What a tree does not record
    ///
    /// **Item order** — files are read back in directory order, so set
    /// [`seq`](Sequence) on items whose order matters — and **comments**, which
    /// serde does not preserve. [`save_item`](Self::save_item) limits the second
    /// to the one file you touched.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        if self.bundled == Some(false) {
            return write_unbundled(self, path);
        }
        write_yaml(path, self)
    }

    /// Re-read a single item from disk, without touching this collection.
    ///
    /// Nothing is spliced in: you get the item as it now exists on disk, with
    /// its [`Source`] set, and decide what to do with it. That is deliberate —
    /// whether an incoming version should replace an item the user has been
    /// editing is the application's call, not this crate's. Match it against
    /// the tree by [`Item::source`] to replace it, or push it to add it.
    ///
    /// `path` may be absolute or relative to the collection root, and may name
    /// either a folder's directory or its `folder.yml` — both give you the
    /// folder, with its subtree.
    ///
    /// This is the reload counterpart to [`save_item`](Self::save_item), and is
    /// meant to be driven by a file watcher. This crate deliberately does not
    /// provide one: delivering events would force a channel or async runtime on
    /// you, and suppressing the events your own [`save`](Self::save) produces
    /// needs to know your save lifecycle.
    ///
    /// Replacing an item is then a `position` and an assignment:
    ///
    /// ```
    /// use opencollection::{Error, HttpRequest, Item, OpenCollection};
    /// # let dir = std::env::temp_dir().join("opencollection-doc-load-item");
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # let mut seed = OpenCollection::new("Petstore")
    /// #     .item(HttpRequest::get("https://example.com/pets").name("List pets"));
    /// # seed.bundled = Some(false);
    /// # seed.save(&dir)?;
    /// # let mut collection = OpenCollection::load(&dir)?;
    /// # let edited = std::fs::read_to_string(dir.join("List pets.yml"))
    /// #     .unwrap()
    /// #     .replace("name: List pets", "name: Renamed on disk");
    /// # std::fs::write(dir.join("List pets.yml"), edited).unwrap();
    ///
    /// // Something told us this path changed.
    /// match collection.load_item("List pets.yml") {
    ///     Ok(fresh) => {
    ///         let items = collection.items.get_or_insert_with(Vec::new);
    ///         match items.iter().position(|item| item.source() == fresh.source()) {
    ///             Some(index) => items[index] = fresh, // changed
    ///             None => items.push(fresh),           // newly appeared
    ///         }
    ///     }
    ///     Err(Error::Deleted { .. }) => { /* removed on disk */ }
    ///     Err(Error::NotAnItem { .. }) => { /* a README, a .proto, a config file */ }
    ///     Err(error) => return Err(error),
    /// }
    ///
    /// assert_eq!(collection.iter().next().unwrap().name(), Some("Renamed on disk"));
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # Ok::<(), opencollection::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Deleted`] if the file is gone — how a deletion reaches you.
    /// - [`Error::NotAnItem`] for a path that is not an item of this
    ///   collection: a `.proto` or README beside the requests, a config file,
    ///   or anything outside the collection directory. Watchers report every
    ///   path, so this is a normal event to ignore rather than a failure.
    /// - [`Error::Parse`] naming the file if it does not parse — an editor may
    ///   simply have saved it mid-edit.
    pub fn load_item(&self, path: impl AsRef<Path>) -> Result<Item, Error> {
        let path = path.as_ref();

        if self.bundled != Some(false) {
            return Err(Error::layout(
                path,
                "a bundled collection keeps its items in one file, so there is \
                 nothing to reload individually: load the collection"
                    .to_owned(),
            ));
        }

        let root = self.source.path().ok_or_else(|| {
            Error::layout(
                path,
                "this collection was not loaded from a directory, so item paths \
                 cannot be resolved against it"
                    .to_owned(),
            )
        })?;

        let mut target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };

        // A folder's config stands in for the folder itself: a watcher reports
        // `pets/folder.yml`, and the item that changed is `pets`.
        let is_config = target.file_stem().and_then(|stem| stem.to_str()) == Some(FOLDER_STEM)
            && matches!(
                target.extension().and_then(|extension| extension.to_str()),
                Some("yml" | "yaml")
            );
        if is_config && let Some(parent) = target.parent() {
            target = parent.to_path_buf();
        }

        if !target.exists() {
            return Err(Error::Deleted { path: target });
        }

        // Canonicalize only as a fallback, so a relative root and an absolute
        // argument still line up. It needs the path to exist, hence the order.
        let relative = target
            .strip_prefix(root)
            .ok()
            .map(Path::to_path_buf)
            .or_else(|| {
                let resolved = target.canonicalize().ok()?;
                let root = root.canonicalize().ok()?;
                resolved.strip_prefix(&root).ok().map(Path::to_path_buf)
            })
            .ok_or_else(|| Error::NotAnItem {
                path: target.clone(),
            })?;

        if target.is_dir() {
            let Some(config) = config_file(&target, FOLDER_STEM) else {
                return Err(Error::NotAnItem { path: target });
            };
            let mut folder: Folder = read_yaml(&config)?;
            let nested = read_items(&target, root, FOLDER_STEM)?;
            if !nested.is_empty() {
                folder.items.get_or_insert_with(Vec::new).extend(nested);
            }
            let mut item = Item::Folder(folder);
            item.set_source(relative);
            return Ok(item);
        }

        // Config files belong to the tree, not to any item. Which stem is
        // reserved depends on the depth the file sits at.
        let reserved = if relative.components().count() > 1 {
            FOLDER_STEM
        } else {
            ROOT_STEM
        };
        let extension = target.extension().and_then(|extension| extension.to_str());
        let stem = target.file_stem().and_then(|stem| stem.to_str());
        if !matches!(extension, Some("yml" | "yaml")) || stem == Some(reserved) {
            return Err(Error::NotAnItem { path: target });
        }

        let mut item: Item = read_yaml(&target)?;
        item.set_source(relative);
        Ok(item)
    }

    /// Write a single item back to the collection at `path`.
    ///
    /// In an unbundled collection this writes just that item's file — and, for
    /// a folder, its `folder.yml` and everything beneath it, so adding a request
    /// to a folder and saving the folder actually persists the request. The root
    /// `opencollection.yml` is rewritten too, but only if its content has
    /// changed, so collection-level edits are not stranded and a file watcher
    /// does not see the root touched on every keystroke.
    ///
    /// In a bundled collection the item's home *is* the whole document, so this
    /// is exactly [`save`](Self::save) — every other pending change lands on
    /// disk with it. That asymmetry is inherent to the single-file layout.
    ///
    /// `path` follows the same rule as [`save`](Self::save): a directory when
    /// unbundled, the collection file when bundled.
    ///
    /// ```
    /// use opencollection::{HttpRequest, Item, OpenCollection};
    /// # let dir = std::env::temp_dir().join("opencollection-doc-save-item");
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # let mut seed = OpenCollection::new("Petstore")
    /// #     .item(HttpRequest::get("https://example.com/pets").name("List pets"))
    /// #     .item(HttpRequest::post("https://example.com/pets").name("Create pet"));
    /// # seed.bundled = Some(false);
    /// # seed.save(&dir)?;
    /// let mut collection = OpenCollection::load(&dir)?;
    ///
    /// let Some(Item::Http(request)) = collection.items.as_mut().unwrap().first_mut() else {
    ///     unreachable!()
    /// };
    /// request.info.as_mut().unwrap().name = Some("Edited".to_owned());
    ///
    /// // Writes `List pets.yml` alone — `Create pet.yml` is not touched.
    /// let item = &collection.items.as_ref().unwrap()[0];
    /// collection.save_item(item, &dir)?;
    /// # assert_eq!(OpenCollection::load(&dir)?.iter().next().unwrap().name(), Some("Edited"));
    /// # let _ = std::fs::remove_dir_all(&dir);
    /// # Ok::<(), opencollection::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The item must belong to this collection and must have been saved before,
    /// so that it has a [`Source`] saying where it lives. A brand-new item has
    /// no home yet — call [`save`](Self::save) once to place it.
    pub fn save_item(&self, item: &Item, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        if self.bundled != Some(false) {
            return self.save(path);
        }

        // Pointer identity, not equality: two requests can be equal while being
        // different items, and writing a foreign item would plant a file that
        // reads back as an item this collection never had.
        if !self.iter().any(|candidate| std::ptr::eq(candidate, item)) {
            return Err(Error::layout(
                path,
                "item does not belong to this collection".to_owned(),
            ));
        }

        let source = item.source().ok_or_else(|| {
            Error::layout(
                path,
                format!(
                    "item `{}` has never been saved, so it has no file yet: \
                     save the whole collection once to place it",
                    item.name().unwrap_or("<unnamed>")
                ),
            )
        })?;

        let target = path.join(source);
        let parent = target.parent().unwrap_or(path);
        if !parent.is_dir() {
            return Err(Error::layout(
                &target,
                "the directory this item lives in does not exist: \
                 save the whole collection to create it"
                    .to_owned(),
            ));
        }

        match item {
            Item::Folder(folder) => {
                std::fs::create_dir_all(&target).map_err(|source| Error::io(&target, source))?;
                let config = Folder {
                    items: None,
                    ..folder.clone()
                };
                write_yaml(&config_path(&target, FOLDER_STEM), &config)?;
                write_items(
                    folder.items.as_deref().unwrap_or_default(),
                    &target,
                    FOLDER_STEM,
                    &mut HashSet::new(),
                )?;
            }
            item => write_yaml(&target, item)?,
        }

        // Rewrite the root only on an actual change.
        let root = OpenCollection {
            items: None,
            ..self.clone()
        };
        let yaml = serde_yaml_ng::to_string(&root)?;
        let root_path = config_path(path, ROOT_STEM);
        if std::fs::read_to_string(&root_path).unwrap_or_default() != yaml {
            std::fs::write(&root_path, yaml).map_err(|error| Error::io(&root_path, error))?;
        }

        Ok(())
    }

    /// Iterate depth-first over every item, descending into folders.
    ///
    /// Folders are yielded too, each before its children, so the count includes
    /// them. Use [`requests`](Self::requests) for just the requests.
    ///
    /// ```
    /// use opencollection::{Folder, HttpRequest, Item, OpenCollection};
    ///
    /// let collection = OpenCollection::new("Petstore").item(
    ///     Folder::new("Pets")
    ///         .item(HttpRequest::get("https://example.com/pets").name("List pets"))
    ///         .item(HttpRequest::post("https://example.com/pets").name("Create pet")),
    /// );
    ///
    /// let names: Vec<_> = collection.iter().filter_map(Item::name).collect();
    /// assert_eq!(names, ["Pets", "List pets", "Create pet"]);
    /// ```
    ///
    /// `&OpenCollection` also implements [`IntoIterator`], so `for item in
    /// &collection` works.
    pub fn iter(&self) -> ItemIter<'_> {
        ItemIter::new(self.items.as_deref().unwrap_or_default())
    }

    /// Iterate over the requests (HTTP, GraphQL, gRPC, WebSocket), descending
    /// into folders.
    ///
    /// Folders and script files are skipped; everything else is a request.
    ///
    /// ```
    /// use opencollection::{Folder, HttpRequest, OpenCollection};
    ///
    /// let collection = OpenCollection::new("Petstore")
    ///     .item(HttpRequest::get("https://example.com/pets").name("List pets"))
    ///     .item(
    ///         Folder::new("Admin")
    ///             .item(HttpRequest::delete("https://example.com/pets/1").name("Delete pet")),
    ///     );
    ///
    /// assert_eq!(collection.iter().count(), 3); // the folder counts
    /// assert_eq!(collection.requests().count(), 2); // it does not
    /// ```
    pub fn requests(&self) -> impl Iterator<Item = &Item> {
        self.iter().filter(|item| item.is_request())
    }
}

impl<'a> IntoIterator for &'a OpenCollection {
    type Item = &'a Item;
    type IntoIter = ItemIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A depth-first iterator over items, descending into folders.
///
/// Yields every [`Item`] including the folders themselves; a folder is
/// yielded before its children.
#[derive(Debug, Clone)]
pub struct ItemIter<'a> {
    stack: Vec<&'a Item>,
}

impl<'a> ItemIter<'a> {
    fn new(items: &'a [Item]) -> Self {
        let mut stack: Vec<&'a Item> = items.iter().collect();
        stack.reverse();
        ItemIter { stack }
    }
}

impl<'a> Iterator for ItemIter<'a> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stack.pop()?;
        if let Item::Folder(folder) = item
            && let Some(children) = &folder.items
        {
            self.stack.extend(children.iter().rev());
        }
        Some(item)
    }
}

impl std::iter::FusedIterator for ItemIter<'_> {}

/// Errors returned by this crate.
///
/// Every variant that can name a file does, because a bare
/// "No such file or directory" is close to useless when a collection is spread
/// over fifty of them.
///
/// The variants worth matching on are [`Deleted`](Error::Deleted) and
/// [`NotAnItem`](Error::NotAnItem): when reloading, they are ordinary outcomes
/// rather than failures.
///
/// ```
/// use opencollection::{Error, OpenCollection};
/// # let dir = std::env::temp_dir().join("opencollection-doc-error");
/// # let _ = std::fs::remove_dir_all(&dir);
/// # std::fs::create_dir_all(&dir).unwrap();
///
/// // A directory with no `opencollection.yml` is not a collection.
/// match OpenCollection::load(&dir) {
///     Err(Error::Layout { path, message }) => {
///         println!("{}: {message}", path.display());
///     }
///     other => panic!("expected a layout error, got {other:?}"),
/// }
/// # let _ = std::fs::remove_dir_all(&dir);
/// ```
#[derive(Debug)]
pub enum Error {
    /// YAML (de)serialization failed.
    Yaml(serde_yaml_ng::Error),
    /// Parsing a file failed.
    ///
    /// Like [`Error::Yaml`], but from a path we know, so an unbundled
    /// collection can name the file at fault instead of reporting a bare
    /// "unknown field" from somewhere in a tree.
    Parse {
        path: PathBuf,
        source: serde_yaml_ng::Error,
    },
    /// Reading or writing a file failed.
    ///
    /// Carries the path so callers can tell *which* file failed; a bare
    /// "No such file or directory" is nearly useless in a file-oriented API.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A collection's storage layout contradicts itself: a directory with no
    /// `opencollection.yml`, or a [`bundled`](OpenCollection::bundled) flag
    /// that disagrees with whether the collection is a file or a tree.
    Layout { path: PathBuf, message: String },
    /// The file backing an item is gone.
    ///
    /// This is how a deletion arrives when reloading: something reports that a
    /// path changed, and by the time [`load_item`](OpenCollection::load_item)
    /// reads it there is nothing there. Distinct from [`Error::Io`] so that
    /// "the item was removed" does not have to be recovered by inspecting an
    /// [`std::io::ErrorKind`].
    Deleted { path: PathBuf },
    /// The path is not an item of this collection.
    ///
    /// A `.proto`, `.env` or README living beside the requests, a config file
    /// (`opencollection.yml`, `folder.yml`) which belongs to the tree rather
    /// than to any item, or a path outside the collection directory entirely.
    NotAnItem { path: PathBuf },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn layout(path: impl Into<PathBuf>, message: String) -> Self {
        Error::Layout {
            path: path.into(),
            message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Yaml(error) => write!(f, "YAML error: {error}"),
            Error::Parse { path, source } => {
                write!(f, "YAML error in {}: {source}", path.display())
            }
            Error::Io { path, source } => write!(f, "I/O error for {}: {source}", path.display()),
            Error::Layout { path, message } => write!(f, "{}: {message}", path.display()),
            Error::Deleted { path } => write!(f, "{}: no longer exists", path.display()),
            Error::NotAnItem { path } => {
                write!(f, "{}: not an item of this collection", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Yaml(error) => Some(error),
            Error::Parse { source, .. } => Some(source),
            Error::Io { source, .. } => Some(source),
            Error::Layout { .. } | Error::Deleted { .. } | Error::NotAnItem { .. } => None,
        }
    }
}

impl From<serde_yaml_ng::Error> for Error {
    fn from(error: serde_yaml_ng::Error) -> Self {
        Error::Yaml(error)
    }
}

// ---------------------------------------------------------------------------
// The unbundled (exploded) filesystem layout.
//
// A bundled collection is a single YAML file. An unbundled one is a directory
// tree: an `opencollection.yml` holding everything *except* the items, one
// `.yml` file per request or script, and one subdirectory per folder carrying
// its own `folder.yml`.
//
//     petstore/
//     ├── opencollection.yml
//     ├── list-pets.yml
//     └── pet-by-owner/
//         ├── folder.yml
//         └── create-pet.yml
//
// The spec fixes those two file names but says nothing about how the rest are
// derived, so the rules live here — see `file_stem` and `unique_stem`.
// ---------------------------------------------------------------------------

/// File name (without extension) of the collection's root config.
const ROOT_STEM: &str = "opencollection";
/// File name (without extension) of a folder's config.
const FOLDER_STEM: &str = "folder";

/// Write `collection` as a directory tree rooted at `dir`.
fn write_unbundled(collection: &OpenCollection, dir: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(dir).map_err(|source| Error::io(dir, source))?;

    // The root file carries everything but the items, which become the tree.
    let root = OpenCollection {
        items: None,
        ..collection.clone()
    };
    write_yaml(&config_path(dir, ROOT_STEM), &root)?;

    let mut written = HashSet::new();
    write_items(
        collection.items.as_deref().unwrap_or_default(),
        dir,
        ROOT_STEM,
        &mut written,
    )?;

    // Only when writing back over the collection we came from: elsewhere we
    // have no basis for deciding a file is stale.
    if same_directory(collection.source.path(), dir) {
        prune(dir, ROOT_STEM, &written)?;
    }

    Ok(())
}

/// Whether `source` and `target` name the same directory on disk.
fn same_directory(source: Option<&Path>, target: &Path) -> bool {
    let Some(source) = source else {
        return false;
    };
    match (source.canonicalize(), target.canonicalize()) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

/// Delete the items under `dir` that this save did not write.
///
/// Scoped to what [`read_items`] would have picked up — a `.yml`/`.yaml` file,
/// or a directory holding a `folder.yml` — so the `.proto`, `.env`, `.pem` and
/// README files that live beside requests are never touched, nor is any
/// directory that is not itself a folder. Config files are left alone too: they
/// belong to the tree, not to any item.
///
/// This is what makes moving and deleting items work. Without it the old file
/// survives the save and the item reappears on the next load.
fn prune(dir: &Path, reserved: &str, written: &HashSet<PathBuf>) -> Result<(), Error> {
    for entry in std::fs::read_dir(dir).map_err(|source| Error::io(dir, source))? {
        let path = entry.map_err(|source| Error::io(dir, source))?.path();

        if path.is_dir() {
            if config_file(&path, FOLDER_STEM).is_none() {
                continue;
            }
            if written.contains(&path) {
                prune(&path, FOLDER_STEM, written)?;
            } else {
                std::fs::remove_dir_all(&path).map_err(|source| Error::io(&path, source))?;
            }
            continue;
        }

        let extension = path.extension().and_then(|extension| extension.to_str());
        let stem = path.file_stem().and_then(|stem| stem.to_str());
        let is_item = matches!(extension, Some("yml" | "yaml")) && stem != Some(reserved);
        if is_item && !written.contains(&path) {
            std::fs::remove_file(&path).map_err(|source| Error::io(&path, source))?;
        }
    }

    Ok(())
}

fn write_items(
    items: &[Item],
    dir: &Path,
    reserved: &str,
    written: &mut HashSet<PathBuf>,
) -> Result<(), Error> {
    for (item, name) in resolve_names(items, reserved) {
        let path = dir.join(&name);
        written.insert(path.clone());

        let Item::Folder(folder) = item else {
            write_yaml(&path, item)?;
            continue;
        };

        std::fs::create_dir_all(&path).map_err(|source| Error::io(&path, source))?;
        let config = Folder {
            items: None,
            ..folder.clone()
        };
        write_yaml(&config_path(&path, FOLDER_STEM), &config)?;
        write_items(
            folder.items.as_deref().unwrap_or_default(),
            &path,
            FOLDER_STEM,
            written,
        )?;
    }

    Ok(())
}

/// Decide the file name for every item in one directory.
///
/// Items keep the name they were loaded under, so that renaming a request does
/// not rename its file — the property that makes load → save → load idempotent.
/// Only items without a usable one get a name derived from their title.
fn resolve_names<'a>(items: &'a [Item], reserved: &str) -> Vec<(&'a Item, String)> {
    // One name space per directory, seeded with the config file's stem so that
    // an item named "folder" cannot land on `folder.yml` and be read back as
    // the folder's own config. Keyed on the stem, so `x.yml` and `x.yaml`
    // cannot both be written and then read back as two separate items.
    //
    // Case-folded, because macOS and Windows filesystems are case-insensitive:
    // `Folder.yml` and `folder.yml` are one file there, so treating them as
    // distinct silently overwrites — a request named "Folder" would land on its
    // own folder's config. Two items whose names differ only in case get a
    // suffix, which is a redundant rename on Linux and data preserved everywhere.
    let mut used = HashSet::from([reserved.to_lowercase()]);

    // Existing names are checked first, so a new item cannot take the file name
    // of one already on disk. This *validates* rather than claims: an item
    // whose name is already spoken for — including one moved in from a folder,
    // where `opencollection` is a legal item name but here belongs to the
    // collection itself — falls through and is renamed below. Writing it as-is
    // would overwrite the config file, and since the root object accepts
    // unknown fields, the next load would silently yield an empty collection.
    let kept: Vec<Option<String>> = items
        .iter()
        .map(|item| {
            let name = item.source()?.file_name()?.to_str()?.to_owned();
            used.insert(stem_of(&name).to_lowercase()).then_some(name)
        })
        .collect();

    items
        .iter()
        .zip(kept)
        .map(|(item, name)| {
            let name = name.unwrap_or_else(|| {
                let stem = unique_stem(&mut used, file_stem(item));
                match item {
                    // A folder's home is its directory, which has no extension.
                    Item::Folder(_) => stem,
                    _ => format!("{stem}.yml"),
                }
            });
            (item, name)
        })
        .collect()
}

/// A file name without its YAML extension. Directory names are returned as-is.
fn stem_of(name: &str) -> &str {
    name.strip_suffix(".yml")
        .or_else(|| name.strip_suffix(".yaml"))
        .unwrap_or(name)
}

/// Read a collection from a directory tree rooted at `dir`.
fn read_unbundled(dir: &Path) -> Result<OpenCollection, Error> {
    let root_path = config_file(dir, ROOT_STEM).ok_or_else(|| {
        Error::layout(
            dir,
            format!(
                "not an unbundled collection: no `{ROOT_STEM}.yml` at the root of the directory"
            ),
        )
    })?;

    let mut collection: OpenCollection = read_yaml(&root_path)?;

    // The tree layout is what `bundled: false` means, so a tree claiming to be
    // bundled is contradicting itself. Reading it anyway would hand back a
    // collection whose flag disagrees with where it came from, and the next
    // write would silently flatten the tree into one file.
    if collection.bundled == Some(true) {
        return Err(Error::layout(
            &root_path,
            "declares `bundled: true` but is stored as a directory tree".to_owned(),
        ));
    }

    // Appending rather than replacing: we never write inline items, so for a
    // tree we produced this just fills an empty slot, but a hand-written or
    // partially bundled root keeps the items it declared.
    let items = read_items(dir, dir, ROOT_STEM)?;
    if !items.is_empty() {
        collection.items.get_or_insert_with(Vec::new).extend(items);
    }

    collection.source = Source::at(dir);
    Ok(collection)
}

/// A collection's or folder's config file, accepting either YAML extension.
fn config_file(dir: &Path, stem: &str) -> Option<PathBuf> {
    ["yml", "yaml"]
        .into_iter()
        .map(|extension| dir.join(format!("{stem}.{extension}")))
        .find(|path| path.is_file())
}

/// Where a config file should be written: over the existing one if there is
/// one, so a tree using `.yaml` does not end up with both extensions.
fn config_path(dir: &Path, stem: &str) -> PathBuf {
    config_file(dir, stem).unwrap_or_else(|| dir.join(format!("{stem}.yml")))
}

/// Collect the items in `dir`, ignoring the config file named by `reserved`.
///
/// Anything that is neither a YAML file nor a directory holding a `folder.yml`
/// is skipped: collections keep `.proto` files, `.env` files and READMEs next
/// to their requests, and those are not items.
fn read_items(dir: &Path, root: &Path, reserved: &str) -> Result<Vec<Item>, Error> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|source| Error::io(dir, source))? {
        paths.push(entry.map_err(|source| Error::io(dir, source))?.path());
    }
    // `read_dir` order is unspecified; sort so a tree reads back the same way
    // on every platform and run.
    paths.sort();

    let mut items = Vec::new();
    for path in paths {
        let mut item = if path.is_dir() {
            let Some(config) = config_file(&path, FOLDER_STEM) else {
                continue;
            };
            let mut folder: Folder = read_yaml(&config)?;
            let nested = read_items(&path, root, FOLDER_STEM)?;
            if !nested.is_empty() {
                folder.items.get_or_insert_with(Vec::new).extend(nested);
            }
            Item::Folder(folder)
        } else {
            let extension = path.extension().and_then(|extension| extension.to_str());
            let stem = path.file_stem().and_then(|stem| stem.to_str());
            if !matches!(extension, Some("yml" | "yaml")) || stem == Some(reserved) {
                continue;
            }
            read_yaml(&path)?
        };

        // Relative to the collection root, so the tree can be written back out
        // under a different directory at any nesting depth.
        item.set_source(path.strip_prefix(root).unwrap_or(&path));
        items.push(item);
    }

    Ok(items)
}

/// The file name an item gets, before collisions are resolved.
///
/// Names are optional throughout the schema — `info` itself is optional on
/// every request type, and [`ScriptFile`] has no name field at all — so falling
/// back to the item's type is mandatory, not a nicety.
fn file_stem(item: &Item) -> String {
    let slug = item.name().map(sanitize_name).unwrap_or_default();
    if !slug.is_empty() {
        return slug;
    }

    match item {
        Item::Http(_) => "http",
        Item::GraphQl(_) => "graphql",
        Item::Grpc(_) => "grpc",
        Item::WebSocket(_) => "websocket",
        Item::Folder(_) => "folder",
        Item::Script(_) => "script",
    }
    .to_owned()
}

/// Turn a title into a file name, replacing only what a filesystem rejects.
///
/// `"Create user"` becomes `"Create user"`, not `"create-user"`. This mirrors
/// `sanitizeName` in Bruno's `utils/common/regex.js` — the same illegal-character
/// set, the same trimming — so files this crate creates sit consistently beside
/// files Bruno creates in the same collection. Slugifying instead would make
/// every collection shared between the two tools visibly inconsistent.
///
/// Returns an empty string when nothing usable survives, which is the signal
/// [`file_stem`] uses to fall back to the item's type.
fn sanitize_name(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            ch if ch.is_control() => '-',
            ch => ch,
        })
        .collect();

    replaced
        .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '-')
        .trim_end_matches(|ch: char| ch.is_whitespace() || ch == '.')
        .to_owned()
}

/// Disambiguate `base` against the names already used in this directory.
///
/// Suffixing only on collision, rather than numbering every item positionally,
/// keeps inserts cheap: adding an item to the top of a folder renames nothing.
/// Clean diffs are the point of the unbundled layout.
fn unique_stem(used: &mut HashSet<String>, base: String) -> String {
    if used.insert(base.to_lowercase()) {
        return base;
    }
    (2u32..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| used.insert(candidate.to_lowercase()))
        .expect("an unused suffix always exists")
}

fn write_yaml<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let yaml = serde_yaml_ng::to_string(value)?;
    std::fs::write(path, yaml).map_err(|source| Error::io(path, source))
}

fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let yaml = std::fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    serde_yaml_ng::from_str(&yaml).map_err(|source| Error::Parse {
        path: PathBuf::from(path),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_discrimination() {
        let yaml = r#"
- info:
    name: Get user
    type: http
  http:
    method: GET
    url: https://example.com/users/1
- info:
    name: Users
    type: folder
  items:
    - info:
        name: nested
      graphql:
        url: https://example.com/graphql
- type: script
  script: console.log("hi");
"#;
        let items: Vec<Item> = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(items[0], Item::Http(_)));
        assert!(matches!(items[1], Item::Folder(_)));
        assert!(matches!(items[2], Item::Script(_)));
        let Item::Folder(folder) = &items[1] else {
            unreachable!()
        };
        assert!(matches!(
            folder.items.as_ref().unwrap()[0],
            Item::GraphQl(_)
        ));
    }

    #[test]
    fn name_only_folder_is_not_a_request() {
        let yaml = "info:\n  name: Just a folder\n  type: folder\n";
        let item: Item = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(item, Item::Folder(_)));
    }

    #[test]
    fn auth_inherit_round_trip() {
        let auth: Auth = serde_yaml_ng::from_str("inherit").unwrap();
        assert_eq!(auth, Auth::Inherit);
        assert_eq!(serde_yaml_ng::to_string(&auth).unwrap().trim(), "inherit");
    }

    #[test]
    fn auth_oauth2_nested_tags() {
        let yaml = "type: oauth2\nflow: authorization_code\naccessTokenUrl: https://example.com/token\npkce:\n  method: S256\n";
        let auth: Auth = serde_yaml_ng::from_str(yaml).unwrap();
        let Auth::OAuth2(AuthOAuth2::AuthorizationCode(flow)) = &auth else {
            panic!("expected authorization code flow, got {auth:?}");
        };
        assert_eq!(flow.pkce.as_ref().unwrap().method, Some(PkceMethod::S256));
        let round_tripped: Auth =
            serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&auth).unwrap()).unwrap();
        assert_eq!(auth, round_tripped);
    }

    #[test]
    fn secret_vs_plain_variables() {
        let yaml = "- name: plain\n  value: v\n- secret: true\n  name: hidden\n";
        let variables: Vec<EnvironmentVariable> = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(matches!(variables[0], EnvironmentVariable::Plain(_)));
        assert!(matches!(variables[1], EnvironmentVariable::Secret(_)));
    }

    #[test]
    fn inheritable_settings() {
        let yaml = "timeout: inherit\nmaxRedirects: 5\n";
        let settings: HttpRequestSettings = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(settings.timeout, Some(Inheritable::Inherit));
        assert_eq!(
            settings.max_redirects,
            Some(Inheritable::Value(Number::Int(5)))
        );
        let round_tripped: HttpRequestSettings =
            serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&settings).unwrap()).unwrap();
        assert_eq!(settings, round_tripped);
    }

    #[test]
    fn method_is_preserved_verbatim() {
        // The schema types `method` as a free string, so casing and custom
        // verbs must survive a round-trip untouched.
        for method in ["GET", "get", "PROPFIND", "PropFind"] {
            let yaml = format!("http:\n  method: {method}\n  url: https://example.com\n");
            let request: HttpRequest = serde_yaml_ng::from_str(&yaml).unwrap();
            assert_eq!(
                request.http.as_ref().unwrap().method.as_deref(),
                Some(method)
            );
            let round_tripped = serde_yaml_ng::to_string(&request).unwrap();
            assert!(round_tripped.contains(&format!("method: {method}")));
        }
    }

    #[test]
    fn unknown_fields_rejected() {
        let yaml = "info:\n  name: x\nhttp:\n  method: GET\n  bogus: true\n";
        assert!(serde_yaml_ng::from_str::<HttpRequest>(yaml).is_err());
    }

    #[test]
    fn description_object_rejects_unknown_fields() {
        // The schema marks the description object `additionalProperties: false`.
        // `deny_unknown_fields` is ignored on variants of an untagged enum, so
        // the object form has to be a named struct for this to hold.
        assert!(
            serde_yaml_ng::from_str::<Description>("content: hi\ntype: text/plain\nbogus: 1\n")
                .is_err()
        );
        let ok: Description = serde_yaml_ng::from_str("content: hi\ntype: text/plain\n").unwrap();
        assert!(matches!(ok, Description::Content(_)));
        // The plain-string form still works.
        assert!(matches!(
            serde_yaml_ng::from_str::<Description>("just text").unwrap(),
            Description::Text(_)
        ));
    }

    #[test]
    fn type_tag_selects_the_item_variant() {
        // A folder with no `items` key is structurally identical to an HTTP
        // request with no `http` key; only the `type` tag tells them apart.
        let folder: Vec<Item> =
            serde_yaml_ng::from_str("- info:\n    name: Empty\n    type: folder\n").unwrap();
        assert!(matches!(folder[0], Item::Folder(_)));

        let request: Vec<Item> =
            serde_yaml_ng::from_str("- info:\n    name: Empty\n    type: http\n").unwrap();
        assert!(matches!(request[0], Item::Http(_)));

        // `type` is optional in the schema, so untagged items are selected by
        // the key that identifies their shape.
        let by_shape: Vec<Item> =
            serde_yaml_ng::from_str("- info:\n    name: Users\n  items: []\n").unwrap();
        assert!(matches!(by_shape[0], Item::Folder(_)));
        let by_shape: Vec<Item> =
            serde_yaml_ng::from_str("- info:\n    name: G\n  graphql:\n    url: u\n").unwrap();
        assert!(matches!(by_shape[0], Item::GraphQl(_)));
    }

    #[test]
    fn ambiguous_items_are_rejected() {
        // The schema defines `Item` as a `oneOf`, so an object carrying neither
        // a `type` tag nor a shape key matches five or six branches at once and
        // is not a valid item. Guessing a variant here would be silently wrong.
        for yaml in ["- info:\n    name: X\n", "- {}\n"] {
            let error = serde_yaml_ng::from_str::<Vec<Item>>(yaml)
                .expect_err("ambiguous item should be rejected")
                .to_string();
            assert!(
                error.contains("cannot tell what kind of item"),
                "got: {error}"
            );
        }
    }

    #[test]
    fn item_errors_name_the_actual_problem() {
        // Untagged matching used to collapse every failure into
        // "data did not match any variant of untagged enum Item".
        let typo = "- info:\n    name: X\n    type: graphql\n  graphql:\n    bogus: 1\n";
        let error = serde_yaml_ng::from_str::<Vec<Item>>(typo)
            .unwrap_err()
            .to_string();
        assert!(error.contains("bogus"), "unhelpful error: {error}");

        let unknown = "- info:\n    name: X\n    type: app\n";
        let error = serde_yaml_ng::from_str::<Vec<Item>>(unknown)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown item type `app`"), "got: {error}");
    }

    #[test]
    fn names_keep_everything_a_filesystem_allows() {
        // Matches `sanitizeName` in Bruno's utils/common/regex.js: only
        // filesystem-illegal characters are replaced, so case and spaces
        // survive and our files look like the ones Bruno writes.
        assert_eq!(sanitize_name("Create user"), "Create user");
        assert_eq!(sanitize_name("List pets (v2)"), "List pets (v2)");
        assert_eq!(sanitize_name("GET /pets/{id}"), "GET -pets-{id}");
        assert_eq!(sanitize_name("a:b|c?d*e"), "a-b-c-d-e");
        assert_eq!(sanitize_name("  spaced out  "), "spaced out");
        assert_eq!(sanitize_name("trailing..."), "trailing");
        // Empty is the signal that `file_stem` should fall back to the type.
        assert_eq!(sanitize_name("///"), "");
        assert_eq!(sanitize_name("   "), "");
    }

    #[test]
    fn provenance_is_invisible_to_equality() {
        // Deliberate, not incidental: `assert_eq!` between a built collection
        // and a loaded one has to hold, or every test comparing the two breaks.
        assert_eq!(Source::at("a/one.yml"), Source::at("b/two.yml"));
        assert_eq!(Source::at("a/one.yml"), Source::default());
    }

    #[test]
    fn collisions_get_numeric_suffixes() {
        let mut used = HashSet::new();
        assert_eq!(unique_stem(&mut used, "get".to_owned()), "get");
        assert_eq!(unique_stem(&mut used, "get".to_owned()), "get-2");
        assert_eq!(unique_stem(&mut used, "get".to_owned()), "get-3");
        assert_eq!(unique_stem(&mut used, "post".to_owned()), "post");
    }

    #[test]
    fn config_file_stems_are_reserved() {
        // An item named "folder" must not be written as `folder.yml`, which
        // would be read back as the enclosing folder's config.
        let mut used = HashSet::from([FOLDER_STEM.to_owned()]);
        assert_eq!(unique_stem(&mut used, "folder".to_owned()), "folder-2");
    }

    #[test]
    fn extensions_preserve_key_order() {
        // `extensions` is free-form, so a round-trip must not reorder keys.
        let yaml = "opencollection: 1.0.0\nextensions:\n  zebra: 1\n  alpha: 2\n  middle: 3\n";
        let collection: OpenCollection = serde_yaml_ng::from_str(yaml).unwrap();
        let round_tripped = serde_yaml_ng::to_string(&collection).unwrap();
        let zebra = round_tripped.find("zebra").unwrap();
        let alpha = round_tripped.find("alpha").unwrap();
        let middle = round_tripped.find("middle").unwrap();
        assert!(zebra < alpha && alpha < middle, "got:\n{round_tripped}");
    }
}
