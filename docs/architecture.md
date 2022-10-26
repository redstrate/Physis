# Architecture

This is a document going over my various design decisions I made
with physis, to onboard future contributors and also point a light
at possible mistakes and what we can do to solve them!

## Goals

I think it's important to go over what problems physis is set up to solve. It's meant to:

* Make patching easier for custom launchers
* Users who want to write one-off tools that interact with the game data (like scrapers)
* Modding tools that want to read and write game data in a safe way

Physis is all about game data, reading, writing and modifying it. However, it's keen to
keep it _safe_ and prevent invalid data from being written, and if there is invalid data
being read - to make it obvious to the developer. So there is a high-level representation
of a file format, which is built on top of our custom parsers:

```rust
#[derive(Debug)]
pub struct IndexEntry {
    pub hash: u64,
    pub data_file_id: u8,
    pub offset: u32,
}

#[binrw]
#[br(little)]
pub struct IndexFile {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    #[br(count = index_header.index_data_size / 16)]
    pub entries: Vec<IndexHashTableEntry>,
}
```

Here's a section of `src/index.rs` that showcases an example of this methodology, headers and other auto-generated data
is hidden by the developer (as there is very little to worry about there anyway) and they only need access to modifying and
reading entries. There is still work to be done to improve upon this, but this is the general idea when writing for
game format parsing.

## Top-level Architecture

There is a bunch of methods to crack open your dat files, but the best way is to `GameData` and `BootData`. These two
structures help parse and ensure boot and game data is valid.

There is helper methods in `GameData` such as `extract(path)` and `exists(path)` which are wrappers around other public
APIs, but make it easier for developers to get started.

When parsing game data, you'll notice many formats only accept a memory buffer, which is just a `Vec<u8>`. These are
passed as references, and are purely non-owning. The purpose of this is because some files are not backed by a file on disk,
but instead are extracted and processed entirely in memory.