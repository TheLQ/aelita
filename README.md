Aelita
==

[![Aelita](etc/11_aelita_uses_her_smarts.png)](https://codelyoko.fandom.com/wiki/Code_Lyoko)

Data Manager Platform covering millions of torrents, site scrapes, photo catalogs, and any large unique data
collections.
Ask "How would you create a small https://archive.org?" managing storage to cataloging to searching to opening on your
desktop.
Also ponder how ephemeral information becomes.

* Manages files as Vault folders. Aelita tracks
  where it came from (torrent, site scrape, manual download with url.txt, etc),
  where it is (poolA, backupB, tape444, someCloud, etc),
  as well as deep metadata like hashes, stat, various tags, and generated scripts.
  Explained more below
* Keeps Filesystem-native "UI" where possible. For example /a-search/videos-mkv-disk2 is "All .mkv videos on disk2"
* qBittorrent Cluster Manager
* Platform for custom site scrapers, richer indexing of large datasets, and more that a queryable filesystem provides

## Concept

### Vault

The smallest filesystem unit is the immutable or versioned `/vault` directory with `./content` from a single
optionally-provided
`./source`. May be a single downloaded file and source url.txt file, torrent data and source .torrent file, 1 season of
a show,
updating synced photos with no source dir, to entire site rips with maintained site ripping code.
They can be moved between storage tiers, saved to tape,
version tracked with integrity checks, and
track known copies to make redundancy decisions.

Vault's may generate `/browse` directories.
For example 1-click script to open video with this audio and subtitle track,
or most-relevant files extracted from a site rip.
References or symlinks content dir.
They do not depend on frontend to work

### XRN and Spaces

Backend list of anything with an XRN id.
A directory (ZFS dataset to 1 torrent's root path), a torrent (linux ISOs category),
a journal entry, or a tape. Think of it as a tag applicable to anything.
We can add rules to apply them, or dashboards that filter for them.

Most things have an ID. Planned filtering will easily identify un-categorized and mis-categorized data

### An immutable raw journal

Aelita does not pre-process responses before journaling.
Meaning fetching `myapp:8000/list_somedata` journals the entire response
before commiting the lookup tables. If a field was skipped initially say for time
it can losslessly be added later.

While none of this is revolutionary, combined and specialized for qBittorrent and ZFS makes a powerful tool that obtains

- A standard unit of disk data. Movable, copyable, and versionable
- Backend and frontend Platform for vastly different models: videos to photo albums to bookmarks to social media
- Immutable data journal enables allows fast feature iteration without data loss and
  without rushing to parse every little input field immediately
- Synthesize immutable views via symlink farms for consumption by users or apps with different needs.
  For example combine collections on different storage, renamed torrent files from same file in different torrents,
  a directory of 1-click play video scripts instead of navigating deeply nested backend storage.
- Traceability from the journal id model to parsing ZFS diffs between [sanoid]() snapshots

As everyone, this started as shell scripts. Needs quickly grew beyond bash.
Custom was the way to go, however a DIY database, proper backend, and frontend is a large investment.
3rd party apps doesn't care about immutable storage, "helpfully" transcoding
or normalizing EXIF tags. Some require a specific folder structure different from mine.
We can choose the best model for us

## Features

- Managing and categorizing 1+ million torrent collections
- qBittorrent shard management
- [ ] Managing and categorizing site scrapes
- [ ] Video Playback from browser to current machine's mpv player
- [ ] Managing and categorizing site scraping
- [ ] Tracking offline Tape backup

Dreams

- [ ] Managing and categorizing mobile + multi-pc browser history
- [ ] Managing and categorizing mobile + multi-pc browser tab sharing for 1000+ tab hoarders
- [ ] Ephemeral social media like and "save for later" history
- [ ] Social media scrape backend
- [ ] ZFS Diffing and archiving from sanoid snapshots
- [ ] ...

