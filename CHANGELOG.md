# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2024-06-30

### Added
- add markdown editing mode

## [0.2.0] - 2024-05-25

### Added
- enable creating items on the same level
- add command for quickly creating an item
- remove removal of spaces in title
- with toggle item
- with backend
- can get actual available roots
- can add items
- server can actually create root and sections
- abstract commander
- with async commands instead of inline mutations phew.
- add command pattern
- allow async function in command
- move core to tui and begin grpc work
- add protos
- update deps
- with basic server

### Fixed
- *(deps)* update rust crate serde to v1.0.203
- *(deps)* update rust crate prost to 0.12.6
- *(deps)* update rust crate prost to 0.12.5
- *(deps)* update rust crate serde to 1.0.202

### Other
- *(deps)* update all dependencies
- *(deps)* update rust crate itertools to 0.13.0
- move unused imports into cfg
- remove unused functions and fix warnings
- remove unused variables
- fix formatting
- refactor out graph created event
- let state use either local or backend
- remove warnings
- remove extra logs

## [0.1.0] - 2024-05-11

### Added
- add filtering
- implement filter
- skipped for now
- can create root
- add color
- it freaking works
- *(redo-ui)* wip
- use cli plan
- add create section
- render sections
- can actually check marks
- add archive
- add more interaction
- tie dialog to graph
- implement command mode
- make highlight be a little more bold
- with input
- version 0.0.0-working-ui
- left right movement done
- show basic json screen
- add logging for tui
- add storage
- add event layer
- hyperlog with basic engine

### Docs
- add some more notes
- add a few more items
- add some todos

### Fixed
- *(deps)* update all dependencies

### Other
- remove unused
- refactor into classic
- add please
- use cli plan instead
- refactor app
- fix test breaking changes
- check can create sections
- make clippy happy
- check off todos
- fix styles
- remove file|
- refactor
- remove unused imports
- into crates
- into crates
- move subcommands aside
