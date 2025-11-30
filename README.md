Aelita
==

[![Aelita](etc/11_aelita_uses_her_smarts.png)](https://codelyoko.fandom.com/wiki/Code_Lyoko)

Data Manager for millions of torrents plus large(-ish) unique data collections.
Ask "How would you manage a small https://archive.org"?
Also ponder how ephemeral information becomes.

- Writes changes to immutable journal log. Commit writes to effectively a lookup database.
- Journal fully replayable to improve data models
- Generic design to support vastly unique data collections and operations,
  but without tedious reimplementation of common tools
- Do useful things beyond pure data categorization of other tools

Supports

- Managing and categorizing 1+ million torrent collections
- [ ] Managing and categorizing site scrapes
- [ ] Video Playback from browser to current machine's mpv player
- [ ] Managing and categorizing site scraping

Dreams

- [ ] Managing and categorizing mobile + multi-pc browser history
- [ ] Managing and categorizing mobile + multi-pc browser tab sharing for 1000+ tab hoarders
- [ ] Ephemeral social media like and "save for later" history
- [ ] Social media scrape backend
- [ ] ...

```raw
Browser History        > Mutation Log > Distilled Database 
Tabs Open/Close Events                  Frontend Site
Reddit Saved
Twitter Saved
Youtube Downloads
Project Tracking
```


