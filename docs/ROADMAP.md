# Roadmap

`pf3-edit` is now at `v1.0.0`: a scoped, public, installable release of a local, keyboard-first editor inspired by the ISPF editing model.

The roadmap below is intentionally product-shaped rather than command-count-shaped. The goal is not to chase every historical ISPF edge case first; it is to grow `pf3-edit` into a stronger daily editor while preserving its host-inspired interaction model.

## Product Principles

- keep the editor keyboard-first and strongly command-driven
- prefer practical-modern ergonomics over strict terminal nostalgia when they conflict
- preserve the recognisable ISPF surface: `Command ===>`, line commands, bounds, scroll modes, and status messaging
- treat `main` as releasable and grow the product through small, verifiable blocks

## v1.1

Theme: adoption, polish, and trust.

This release line should improve the public usability of `pf3-edit` without redefining the product.

Priority themes:

- better installation reach
  - `cargo install --git` remains documented
  - release archives stay healthy
  - optional packaging follow-ups such as Homebrew can be explored
- editor polish
  - tighter messaging in edge cases
  - small consistency gaps in search, replace, text entry, excludes, and bounds
  - improved command history quality, for example prefix-aware history search
- public-facing quality
  - stronger demos, screenshots, or GIFs
  - clearer supported-platform story
  - more regression tests for real-world workflows

Success signal:

- a new user can install `pf3-edit`, open a file, discover the key editing model quickly, and trust its core save/search/edit flows

## v2.0

Theme: from strong editor to small workbench.

`v2.0` is the first major product expansion. The editor should stop feeling like a single isolated screen and start feeling like a small environment.

Priority themes:

- file and member navigation
  - directory or file-list workflow
  - open and switch flows that still feel ISPF-like
- multiple working surfaces
  - better use of split and swap
  - multiple buffers or multiple views
  - lightweight browse/help panels
- persistent profiles
  - remember settings such as `BOUNDS`, `NUMBER`, `CAPS`, scroll mode, and insert/overwrite mode
- stronger navigation
  - better locate variants
  - labels and special-line navigation
  - smoother movement across larger files and repeated workflows

Success signal:

- `pf3-edit` becomes a credible daily-driver workbench rather than just a powerful single-file screen

## v3.0

Theme: extensibility and power-user leverage.

`v3.0` is where the product becomes meaningfully programmable.

Priority themes:

- macros
  - start with named command macros or scriptable editor actions
  - avoid fragile “record every keystroke” approaches as the first design
- scriptability
  - a modern scripting layer such as Lua or Rhai
  - access to buffer, cursor, line commands, search, and editor state
- extension points
  - user-defined commands
  - extension-friendly panels or workflows
- stronger structured editing
  - richer text and block manipulation beyond the current baseline

Success signal:

- advanced users can teach `pf3-edit` new behaviors instead of waiting for the core product to grow every capability itself

## Desktop Edition

This is a deliberate side path, not the default roadmap spine.

The recommended approach is:

1. keep the TUI as the core product
2. if a desktop path becomes attractive, start with a desktop shell around the existing editor
3. avoid rebuilding the entire interaction model as a GUI too early

Why:

- the current core is already strong and portable
- a desktop shell can add packaging, window management, and discoverability without discarding the TUI architecture
- a full GUI rewrite would be a separate product chapter, not just a release feature

Most plausible desktop directions:

- desktop wrapper around the existing terminal editor
- richer file browser and launcher experience
- optional future hybrid mode with host-like panels plus native shell features

## Not Planned As Core v2/v3 Requirements

These may become interesting later, but they are not currently the defining milestones:

- full z/OS dataset simulation
- REXX compatibility
- strict 3270 authenticity over usability
- broad historical command parity before product-level improvements

## Current Recommendation

The strongest immediate path after `v1.0.0` is:

1. `v1.1` polish and adoption work
2. `v2.0` workbench expansion
3. `v3.0` macros and scriptability

That order keeps the current release useful, grows public value next, and only then adds the deepest extensibility layer.
