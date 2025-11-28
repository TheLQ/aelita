#![allow(non_camel_case_types)]
use chrono::NaiveDateTime;

struct TxDB_Mutation {
    published: NaiveDateTime,
    description: String,
    mut_type: TxDB_MutType,
    // json encoded
    data: TxDb_RedditSave,
}

enum TxDB_MutType {
    FirefoxHistory,
}

struct TxDb_Counters {
    name: String,
    count: u32,
}

struct TxDb_RedditSave {
    version: u8,
    // u16 is "only" 65k comments
    comments_num: u32,
    title: String,
    url: String,
    author: String,
    post_type: String,
    publish_date: NaiveDateTime,
}

struct TxDb_HackerNews {
    version: u8,
    // u16 is "only" 65k comments
    comments_num: u32,
    title: String,
    url: String,
    author: String,
    publish_date: NaiveDateTime,
}

struct TxDb_FirefoxHistory {
    version: u8,
    is_initial_import: bool,
    url: String,
    visit_date: String,
}

//
// --- planned ---
//

struct TxDb_YoutubeVideo {
    version: u8,
    ytid: String,
    vidtype: TxDb_YoutubeType,
    title: String,
}

enum TxDb_YoutubeType {
    LikedVideo,
    History,
    HoloScrape,
}

//
// --- future ---
//

struct TxDb_TwitterLikes {}

struct TxDb_PixivLike {}
