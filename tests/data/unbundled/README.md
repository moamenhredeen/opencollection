# Kitchen Sink (unbundled)

The same collection as `tests/data/full.yml`, stored as a tree.

Hand-authored on purpose: the file names use a numeric-prefix convention that
the writer would never produce, one item uses the `.yaml` extension, and the
`protos/` and `certs/` directories hold non-item files that sit beside the
requests. Reading this back must still yield the bundled fixture.
