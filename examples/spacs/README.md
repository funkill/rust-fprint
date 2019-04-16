# Simple PAC System

This is example of using different data storage for fprint.
We save to database fingerprints with user ids and then using them for identity. For identity we create array of
fingerprints and receive this array to fprint. After scanning, fprint receive offset of fingerprint in this array.

## Using

Before first launch you must create database.

```sh
$ touch fingers.sqlite
$ sqlite3 fingers.sqlite db.sql
```

After that you can save fingers (set `<user_id>`, with one user id you can save many fingerprints)/

```sh
$ cargo run --bin saver -- <user_id>
```

For identity you can use identifier, what sad user id for fingerprint

```sh
$ cargo run --bin identifier
```
