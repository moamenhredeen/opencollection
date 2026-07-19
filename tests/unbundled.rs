//! Reading and writing the unbundled (directory tree) layout.

use std::path::{Path, PathBuf};

mod common;

use common::TempDir;
use opencollection::{Error, Folder, HttpRequest, Item, OpenCollection, ScriptFile, ScriptTypeTag};

/// Paths of every file under `dir`, relative to it and sorted.
fn tree(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("directory should be readable")
            .map(|entry| entry.expect("entry should be readable").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let relative = path.strip_prefix(base).expect("path is under base");
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn unbundled(collection: OpenCollection) -> OpenCollection {
    OpenCollection {
        bundled: Some(false),
        ..collection
    }
}

/// The same collection as `tests/data/full.yml`, stored as a tree.
///
/// Hand-authored rather than produced by the writer: the file names follow a
/// numeric-prefix convention the writer would never emit, one item uses the
/// `.yaml` extension, and `protos/`, `certs/`, `.env.local` and a README sit
/// beside the requests. Every other test here writes a tree before reading it,
/// so they only prove the reader and writer agree with each other — this one
/// holds the reader to a layout it did not produce.
const FIXTURE: &str = "tests/data/unbundled";

#[test]
fn the_fixture_tree_parses_to_the_bundled_fixture() {
    let tree = OpenCollection::load(FIXTURE).expect("fixture tree should parse");
    let bundled = OpenCollection::load("tests/data/full.yml").expect("fixture should parse");

    // The two fixtures are the same collection in the two layouts, so they
    // differ only in the flag that says which layout they are.
    assert_eq!(tree.bundled, Some(false));
    assert_eq!(bundled.bundled, Some(true));
    assert_eq!(
        OpenCollection {
            bundled: None,
            ..tree
        },
        OpenCollection {
            bundled: None,
            ..bundled
        },
    );
}

#[test]
fn the_fixture_tree_preserves_declaration_order() {
    let tree = OpenCollection::load(FIXTURE).unwrap();

    // Order comes from the file names, which is why the fixture numbers them.
    let names: Vec<_> = tree.iter().filter_map(Item::name).collect();
    assert_eq!(
        names,
        [
            "List pets",
            "Create pet",
            "Pet by owner",
            "Owner pets (GraphQL)",
            "Owner pets variants (GraphQL)",
            "Watch pets (gRPC)",
            "Simple gRPC",
            "Pet events (WebSocket)",
            "Single message WS",
            "Auth zoo",
            "awsv4",
            "digest",
            "oauth1",
            "oauth2 password",
            "oauth2 authorization code",
            "oauth2 implicit",
        ]
    );
}

#[test]
fn the_fixture_tree_ignores_the_files_that_are_not_items() {
    let tree = OpenCollection::load(FIXTURE).unwrap();

    // `protos/` and `certs/` have no `folder.yml`, and the README, `.proto`,
    // `.pem` and `.env.local` files are not YAML — none of them are items.
    assert_eq!(tree.iter().count(), 17);
    assert_eq!(tree.requests().count(), 14);
}

#[test]
fn tree_round_trips() {
    let temp = TempDir::new("roundtrip");
    let dir = temp.join("petstore");

    // Names are chosen so directory order matches declaration order; a tree
    // does not record item order, which is what `seq` is for.
    let collection = unbundled(
        OpenCollection::new("Petstore")
            .summary("Example")
            .item(HttpRequest::get("https://example.com/pets").name("A list pets"))
            .item(
                Folder::new("B pets")
                    .item(HttpRequest::post("https://example.com/pets").name("Create pet"))
                    .item(HttpRequest::delete("https://example.com/pets/1").name("Delete pet")),
            ),
    );

    collection.save(&dir).expect("tree should be writable");

    assert_eq!(
        tree(&dir),
        [
            "A list pets.yml",
            "B pets/Create pet.yml",
            "B pets/Delete pet.yml",
            "B pets/folder.yml",
            "opencollection.yml",
        ]
    );

    let reparsed = OpenCollection::load(&dir).expect("tree should be readable");
    assert_eq!(collection, reparsed);
}

#[test]
fn root_file_holds_everything_but_the_items() {
    let temp = TempDir::new("root");
    let dir = temp.join("collection");

    unbundled(OpenCollection::new("Petstore").item(HttpRequest::get("u").name("List pets")))
        .save(&dir)
        .expect("tree should be writable");

    let root = std::fs::read_to_string(dir.join("opencollection.yml")).unwrap();
    assert!(root.contains("name: Petstore"), "got:\n{root}");
    assert!(root.contains("bundled: false"), "got:\n{root}");
    assert!(
        !root.contains("items"),
        "items belong in the tree, not the root file:\n{root}"
    );
}

#[test]
fn nameless_items_fall_back_to_their_type() {
    let temp = TempDir::new("nameless");
    let dir = temp.join("collection");

    // `info` is optional on every request and `ScriptFile` has no name field
    // at all, so the fallback is mandatory rather than a nicety.
    let collection = unbundled(
        OpenCollection::new("Nameless")
            .item(HttpRequest::get("https://example.com/one"))
            .item(HttpRequest::get("https://example.com/two"))
            .item(ScriptFile {
                item_type: Some(ScriptTypeTag::Script),
                script: Some("console.log('hi');".to_owned()),
                ..ScriptFile::default()
            }),
    );

    collection.save(&dir).unwrap();

    assert_eq!(
        tree(&dir),
        ["http-2.yml", "http.yml", "opencollection.yml", "script.yml"]
    );
    assert_eq!(OpenCollection::load(&dir).unwrap().iter().count(), 3);
}

#[test]
fn duplicate_names_get_numeric_suffixes() {
    let temp = TempDir::new("duplicates");
    let dir = temp.join("collection");

    unbundled(
        OpenCollection::new("Dupes")
            .item(HttpRequest::get("https://example.com/1").name("Get user"))
            .item(HttpRequest::get("https://example.com/2").name("get user"))
            .item(HttpRequest::get("https://example.com/3").name("GET USER!")),
    )
    .save(&dir)
    .unwrap();

    // "Get user" and "get user" differ only in case, and on a case-insensitive
    // filesystem they are one file — so the second is suffixed rather than
    // silently overwriting the first. "GET USER!" is a distinct name and keeps it.
    assert_eq!(
        tree(&dir),
        [
            "GET USER!.yml",
            "Get user.yml",
            "get user-2.yml",
            "opencollection.yml",
        ]
    );
}

#[test]
fn config_file_names_are_not_stolen_by_items() {
    let temp = TempDir::new("reserved");
    let dir = temp.join("collection");

    // An item named "folder" inside a folder would otherwise be written as
    // `folder.yml` and read back as that folder's own config.
    let collection = unbundled(
        OpenCollection::new("Reserved").item(
            Folder::new("Outer")
                .item(HttpRequest::get("https://example.com").name("Folder"))
                .item(HttpRequest::get("https://example.com").name("Opencollection")),
        ),
    );

    collection.save(&dir).unwrap();

    assert_eq!(
        tree(&dir),
        [
            "Outer/Folder-2.yml",
            "Outer/Opencollection.yml",
            "Outer/folder.yml",
            "opencollection.yml",
        ]
    );

    let reparsed = OpenCollection::load(&dir).unwrap();
    assert_eq!(collection, reparsed);
    let names: Vec<_> = reparsed.iter().filter_map(Item::name).collect();
    assert_eq!(names, ["Outer", "Folder", "Opencollection"]);
}

#[test]
fn non_yaml_files_are_ignored() {
    let temp = TempDir::new("stray");
    let dir = temp.join("collection");

    unbundled(OpenCollection::new("Stray").item(HttpRequest::get("u").name("List pets")))
        .save(&dir)
        .unwrap();

    // Collections keep proto files, dotenv files and docs beside their
    // requests; none of those are items.
    std::fs::write(dir.join("README.md"), "# notes").unwrap();
    std::fs::write(dir.join("service.proto"), "syntax = \"proto3\";").unwrap();
    std::fs::write(dir.join(".env"), "TOKEN=x").unwrap();
    std::fs::create_dir(dir.join("protos")).unwrap();
    std::fs::write(dir.join("protos/a.proto"), "").unwrap();

    let collection = OpenCollection::load(&dir).unwrap();
    let names: Vec<_> = collection.iter().filter_map(Item::name).collect();
    assert_eq!(names, ["List pets"]);
}

#[test]
fn bundled_collections_still_write_to_a_single_file() {
    let temp = TempDir::new("bundled");
    let path = temp.join("collection.yml");

    let collection = OpenCollection::new("Petstore")
        .item(Folder::new("Pets").item(HttpRequest::get("u").name("List pets")));

    collection.save(&path).unwrap();
    assert!(path.is_file());
    assert_eq!(OpenCollection::load(&path).unwrap(), collection);
}

#[test]
fn a_directory_without_a_root_file_is_rejected() {
    let temp = TempDir::new("no-root");
    std::fs::write(temp.join("stray.yml"), "info:\n  name: x\n").unwrap();

    let error = OpenCollection::load(&temp.0)
        .expect_err("a directory with no opencollection.yml is not a collection");
    assert!(matches!(error, Error::Layout { .. }), "got: {error:?}");
    assert!(
        error.to_string().contains("opencollection.yml"),
        "got: {error}"
    );
}

#[test]
fn a_tree_claiming_to_be_bundled_is_rejected() {
    let temp = TempDir::new("contradiction");
    std::fs::write(
        temp.join("opencollection.yml"),
        "info:\n  name: x\nbundled: true\n",
    )
    .unwrap();

    let error = OpenCollection::load(&temp.0).expect_err("a tree cannot declare itself bundled");
    assert!(error.to_string().contains("bundled: true"), "got: {error}");
}

#[test]
fn a_tree_root_read_as_a_single_file_points_at_the_directory() {
    let temp = TempDir::new("root-file");
    let dir = temp.join("collection");

    unbundled(OpenCollection::new("Petstore").item(HttpRequest::get("u").name("List pets")))
        .save(&dir)
        .unwrap();

    // Reading just the root file would succeed and hand back a collection with
    // no items — an empty result that looks like success.
    let error = OpenCollection::load(dir.join("opencollection.yml"))
        .expect_err("the root file of a tree is not a bundled collection");
    assert!(error.to_string().contains("bundled: false"), "got: {error}");
    assert!(
        error.to_string().contains("collection"),
        "the error should name the directory to read instead: {error}"
    );
}

#[test]
fn parse_errors_name_the_offending_file() {
    let temp = TempDir::new("bad-yaml");
    std::fs::write(temp.join("opencollection.yml"), "info:\n  name: x\n").unwrap();
    std::fs::write(
        temp.join("broken.yml"),
        "info:\n  name: y\nhttp:\n  bogus: 1\n",
    )
    .unwrap();

    let error = OpenCollection::load(&temp.0).expect_err("the item does not parse");
    assert!(matches!(error, Error::Parse { .. }), "got: {error:?}");
    assert!(error.to_string().contains("broken.yml"), "got: {error}");
    assert!(error.to_string().contains("bogus"), "got: {error}");
}

// ---------------------------------------------------------------------------
// Provenance: items are written back to the file they were loaded from.
//
// The regression these guard against was measured, not hypothetical: before
// items carried a source, loading the fixture and saving it wrote a second set
// of files beside the originals, and the collection went from 17 items to 34.
// ---------------------------------------------------------------------------

/// The fixture copied somewhere writable, since saving now prunes.
fn fixture_copy(label: &str) -> (TempDir, PathBuf) {
    let temp = TempDir::new(label);
    let dir = temp.join("collection");
    common::copy_dir(Path::new(FIXTURE), &dir);
    (temp, dir)
}

#[test]
fn resaving_a_loaded_tree_changes_nothing() {
    let (_temp, dir) = fixture_copy("resave");

    let before = OpenCollection::load(&dir).unwrap();
    let files_before = tree(&dir);
    before.save(&dir).unwrap();

    assert_eq!(
        tree(&dir),
        files_before,
        "no file should be added or renamed"
    );
    assert_eq!(OpenCollection::load(&dir).unwrap().iter().count(), 17);
}

#[test]
fn renaming_an_item_does_not_rename_its_file() {
    let (_temp, dir) = fixture_copy("rename");

    let mut collection = OpenCollection::load(&dir).unwrap();
    let Some(Item::Http(request)) = collection.items.as_mut().unwrap().first_mut() else {
        panic!("first item should be an HTTP request");
    };
    request.info.as_mut().unwrap().name = Some("Something else entirely".to_owned());

    collection.save(&dir).unwrap();

    // The display name moved; the file did not. This is Bruno's model: renaming
    // the title and renaming the file are separate operations.
    assert!(dir.join("01-list-pets.yml").is_file(), "{:?}", tree(&dir));
    assert!(!dir.join("Something else entirely.yml").exists());
    let reloaded = OpenCollection::load(&dir).unwrap();
    assert_eq!(reloaded.iter().count(), 17);
    assert_eq!(
        reloaded.iter().next().unwrap().name(),
        Some("Something else entirely")
    );
}

#[test]
fn deleting_an_item_removes_its_file_and_nothing_else() {
    let (_temp, dir) = fixture_copy("delete");

    let mut collection = OpenCollection::load(&dir).unwrap();
    collection.items.as_mut().unwrap().remove(1); // "Create pet"
    collection.save(&dir).unwrap();

    assert!(
        !dir.join("02-create-pet.yml").exists(),
        "the item's file is pruned"
    );
    assert_eq!(OpenCollection::load(&dir).unwrap().iter().count(), 16);

    // Everything that is not an item survives.
    for kept in [
        "README.md",
        ".env.local",
        "protos/pets.proto",
        "certs/cert.pem",
    ] {
        assert!(dir.join(kept).is_file(), "{kept} should survive a save");
    }
}

#[test]
fn moving_an_item_between_folders_leaves_no_copy_behind() {
    let (_temp, dir) = fixture_copy("move");

    let mut collection = OpenCollection::load(&dir).unwrap();
    let moved = collection.items.as_mut().unwrap().remove(0); // "List pets"
    let Some(Item::Folder(folder)) = collection.items.as_mut().unwrap().get_mut(1) else {
        panic!("expected the \"Pet by owner\" folder");
    };
    folder.items.as_mut().unwrap().push(moved);

    collection.save(&dir).unwrap();

    assert!(
        !dir.join("01-list-pets.yml").exists(),
        "the old file is gone"
    );
    assert_eq!(
        OpenCollection::load(&dir).unwrap().iter().count(),
        17,
        "moved, not duplicated"
    );
}

#[test]
fn saving_into_a_different_directory_never_prunes() {
    let (_temp, dir) = fixture_copy("elsewhere");
    let elsewhere = TempDir::new("elsewhere-target");
    let target = elsewhere.join("copy");

    let mut collection = OpenCollection::load(&dir).unwrap();
    collection.items.as_mut().unwrap().remove(1);

    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(
        target.join("stranger.yml"),
        "info:\n  name: x\ntype: http\n",
    )
    .unwrap();
    collection.save(&target).unwrap();

    // We have no basis for calling a file in someone else's directory stale.
    assert!(target.join("stranger.yml").is_file());
    // Relative sources still place nested items correctly under the new root.
    assert!(
        target.join("03-pet-by-owner/01-owner-pets.yml").is_file(),
        "{:?}",
        tree(&target)
    );
}

#[test]
fn an_item_cannot_overwrite_the_root_config() {
    let temp = TempDir::new("clobber");
    let dir = temp.join("collection");
    std::fs::create_dir_all(dir.join("pets")).unwrap();
    std::fs::write(
        dir.join("opencollection.yml"),
        "info:\n  name: Kitchen\nbundled: false\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("pets/folder.yml"),
        "info:\n  name: Pets\n  type: folder\n",
    )
    .unwrap();
    // Legal inside a folder, where the reserved stem is `folder`, not
    // `opencollection` — the existing reserved-name test locks that in.
    std::fs::write(
        dir.join("pets/opencollection.yml"),
        "info:\n  name: Sneaky\n  type: http\nhttp:\n  method: GET\n  url: u\n",
    )
    .unwrap();

    let mut collection = OpenCollection::load(&dir).unwrap();
    let Some(Item::Folder(folder)) = collection.items.as_mut().unwrap().first_mut() else {
        panic!("expected the folder");
    };
    let sneaky = folder.items.as_mut().unwrap().pop().unwrap();
    assert_eq!(
        sneaky.source().unwrap(),
        Path::new("pets/opencollection.yml")
    );

    // Moving it to the root puts its remembered file name on a collision course
    // with the collection's own config. Honouring the source here would write a
    // request over opencollection.yml — and because the root object accepts
    // unknown fields, the next load would silently return an empty collection.
    collection.items.as_mut().unwrap().push(sneaky);
    collection.save(&dir).unwrap();

    let reloaded = OpenCollection::load(&dir).unwrap();
    assert_eq!(
        reloaded.info.as_ref().unwrap().name.as_deref(),
        Some("Kitchen"),
        "the root config must survive"
    );
    let mut names: Vec<_> = reloaded.iter().filter_map(Item::name).collect();
    names.sort_unstable();
    assert_eq!(names, ["Pets", "Sneaky"], "and the item survives, renamed");
}

#[test]
fn a_folder_config_named_yaml_is_recognised() {
    let temp = TempDir::new("folder-yaml");
    let dir = temp.join("collection");
    std::fs::create_dir_all(dir.join("pets")).unwrap();
    std::fs::write(
        dir.join("opencollection.yml"),
        "info:\n  name: X\nbundled: false\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("pets/folder.yaml"),
        "info:\n  name: Pets\n  type: folder\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("pets/get.yml"),
        "info:\n  name: Get\n  type: http\nhttp:\n  method: GET\n  url: u\n",
    )
    .unwrap();

    // Probing only `folder.yml` made the directory invisible, taking its whole
    // subtree with it, silently.
    let collection = OpenCollection::load(&dir).unwrap();
    let names: Vec<_> = collection.iter().filter_map(Item::name).collect();
    assert_eq!(names, ["Pets", "Get"]);
}

#[test]
fn save_item_writes_one_file_and_leaves_the_rest_alone() {
    let (_temp, dir) = fixture_copy("save-item");

    let mut collection = OpenCollection::load(&dir).unwrap();
    let untouched = std::fs::metadata(dir.join("02-create-pet.yml"))
        .unwrap()
        .modified()
        .unwrap();

    let Some(Item::Http(request)) = collection.items.as_mut().unwrap().first_mut() else {
        panic!("first item should be an HTTP request");
    };
    request.info.as_mut().unwrap().name = Some("Edited".to_owned());
    let item = &collection.items.as_ref().unwrap()[0];

    collection.save_item(item, &dir).unwrap();

    assert_eq!(
        OpenCollection::load(&dir)
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .name(),
        Some("Edited")
    );
    assert_eq!(
        std::fs::metadata(dir.join("02-create-pet.yml"))
            .unwrap()
            .modified()
            .unwrap(),
        untouched,
        "a sibling request must not be rewritten"
    );
}

#[test]
fn save_item_on_a_folder_writes_its_children() {
    let (_temp, dir) = fixture_copy("save-folder");

    let mut collection = OpenCollection::load(&dir).unwrap();
    let Some(Item::Folder(folder)) = collection.items.as_mut().unwrap().get_mut(2) else {
        panic!("expected a folder");
    };
    folder.items.as_mut().unwrap().push(
        HttpRequest::get("https://example.com/new")
            .name("Brand new")
            .into(),
    );
    let item = &collection.items.as_ref().unwrap()[2];

    collection.save_item(item, &dir).unwrap();

    assert!(
        dir.join("03-pet-by-owner/Brand new.yml").is_file(),
        "{:?}",
        tree(&dir)
    );
    assert_eq!(OpenCollection::load(&dir).unwrap().iter().count(), 18);
}

#[test]
fn save_item_rejects_items_with_no_file_and_foreign_items() {
    let (_temp, dir) = fixture_copy("save-item-errors");
    let collection = OpenCollection::load(&dir).unwrap();

    let orphan = Item::Http(HttpRequest::get("u").name("Never saved"));
    let error = collection.save_item(&orphan, &dir).unwrap_err();
    assert!(matches!(error, Error::Layout { .. }), "got: {error:?}");
    assert!(
        error.to_string().contains("does not belong"),
        "got: {error}"
    );
}

#[test]
fn save_item_in_bundled_mode_writes_the_whole_document() {
    let temp = TempDir::new("save-item-bundled");
    let path = temp.join("collection.yml");

    let collection = OpenCollection::new("Bundled")
        .item(HttpRequest::get("u").name("One"))
        .item(HttpRequest::get("u").name("Two"));
    collection.save(&path).unwrap();

    let item = &collection.items.as_ref().unwrap()[0];
    collection.save_item(item, &path).unwrap();

    // The item's home is the whole file, so everything lands.
    assert_eq!(OpenCollection::load(&path).unwrap().iter().count(), 2);
}

// ---------------------------------------------------------------------------
// Reloading one item. The library parses; the caller decides what to do with
// the result — including whether it may replace something being edited.
// ---------------------------------------------------------------------------

#[test]
fn load_item_reads_one_request_from_disk() {
    let (_temp, dir) = fixture_copy("load-item");
    let collection = OpenCollection::load(&dir).unwrap();

    // Simulate an external edit — another editor, a git pull.
    let file = dir.join("01-list-pets.yml");
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("name: List pets", "name: Edited elsewhere");
    std::fs::write(&file, edited).unwrap();

    let item = collection.load_item("01-list-pets.yml").unwrap();
    assert_eq!(item.name(), Some("Edited elsewhere"));
    assert_eq!(item.source().unwrap(), Path::new("01-list-pets.yml"));

    // Nothing was spliced in: the collection is untouched until the caller says so.
    assert_eq!(collection.iter().next().unwrap().name(), Some("List pets"));
}

#[test]
fn load_item_accepts_absolute_paths_and_folder_configs() {
    let (_temp, dir) = fixture_copy("load-item-paths");
    let collection = OpenCollection::load(&dir).unwrap();

    let absolute = collection
        .load_item(dir.join("01-list-pets.yml"))
        .expect("an absolute path should resolve against the collection root");
    assert_eq!(absolute.name(), Some("List pets"));

    // A watcher reports `folder.yml`; the item that changed is the folder.
    let by_config = collection.load_item("03-pet-by-owner/folder.yml").unwrap();
    let by_dir = collection.load_item("03-pet-by-owner").unwrap();
    assert_eq!(by_config.name(), Some("Pet by owner"));
    assert_eq!(by_config, by_dir);
    assert_eq!(by_dir.source().unwrap(), Path::new("03-pet-by-owner"));

    // Folders come back with their subtree.
    let Item::Folder(folder) = by_dir else {
        panic!("expected a folder");
    };
    assert_eq!(folder.items.as_ref().unwrap().len(), 6);
}

#[test]
fn load_item_reports_a_deleted_file_as_deleted() {
    let (_temp, dir) = fixture_copy("load-item-deleted");
    let collection = OpenCollection::load(&dir).unwrap();

    std::fs::remove_file(dir.join("01-list-pets.yml")).unwrap();
    let error = collection.load_item("01-list-pets.yml").unwrap_err();
    assert!(matches!(error, Error::Deleted { .. }), "got: {error:?}");

    // A removed folder reports the same way, whichever path the watcher gives.
    std::fs::remove_dir_all(dir.join("03-pet-by-owner")).unwrap();
    for path in ["03-pet-by-owner", "03-pet-by-owner/folder.yml"] {
        let error = collection.load_item(path).unwrap_err();
        assert!(matches!(error, Error::Deleted { .. }), "{path}: {error:?}");
    }
}

#[test]
fn load_item_rejects_paths_that_are_not_items() {
    let (_temp, dir) = fixture_copy("load-item-not-items");
    let collection = OpenCollection::load(&dir).unwrap();

    // Everything a watcher will report that is not an item: files beside the
    // requests, the collection's own config, and a directory that is not a
    // folder. Callers ignore these rather than treating them as failures.
    for path in [
        "README.md",
        ".env.local",
        "protos/pets.proto",
        "protos",
        "opencollection.yml",
    ] {
        let error = collection.load_item(path).unwrap_err();
        assert!(
            matches!(error, Error::NotAnItem { .. }),
            "{path}: {error:?}"
        );
    }

    let outside = collection.load_item("/etc/hosts").unwrap_err();
    assert!(
        matches!(outside, Error::NotAnItem { .. }),
        "got: {outside:?}"
    );
}

#[test]
fn load_item_needs_an_unbundled_collection_loaded_from_disk() {
    let temp = TempDir::new("load-item-bundled");
    let path = temp.join("collection.yml");
    let bundled = OpenCollection::new("Bundled").item(HttpRequest::get("u").name("One"));
    bundled.save(&path).unwrap();

    let error = OpenCollection::load(&path)
        .unwrap()
        .load_item("anything.yml")
        .unwrap_err();
    assert!(error.to_string().contains("one file"), "got: {error}");

    // Built in memory, never loaded: there is no root to resolve against.
    let error = unbundled(OpenCollection::new("Fresh"))
        .load_item("anything.yml")
        .unwrap_err();
    assert!(matches!(error, Error::Layout { .. }), "got: {error:?}");
}

#[test]
fn a_reloaded_item_can_be_put_back_and_saved() {
    let (_temp, dir) = fixture_copy("load-item-splice");
    let mut collection = OpenCollection::load(&dir).unwrap();

    let file = dir.join("02-create-pet.yml");
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("name: Create pet", "name: Externally renamed");
    std::fs::write(&file, edited).unwrap();

    // The caller's job: match on source, replace, carry on.
    let fresh = collection.load_item("02-create-pet.yml").unwrap();
    let items = collection.items.as_mut().unwrap();
    let index = items
        .iter()
        .position(|item| item.source() == fresh.source())
        .expect("the reloaded item should match one in the tree");
    items[index] = fresh;

    assert_eq!(collection.iter().count(), 17);
    collection.save(&dir).unwrap();

    let reloaded = OpenCollection::load(&dir).unwrap();
    assert_eq!(reloaded.iter().count(), 17);
    assert!(
        dir.join("02-create-pet.yml").is_file(),
        "the file keeps its name"
    );
    let names: Vec<_> = reloaded.iter().filter_map(Item::name).collect();
    assert!(names.contains(&"Externally renamed"), "got: {names:?}");
}
