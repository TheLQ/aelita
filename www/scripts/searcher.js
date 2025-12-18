"use strict";

const ID_SEARCH_BOX = "search-box";
const ID_X_ENTRY = "x-entry";
const ID_X_ENTRY_HIDDEN = "x-entry-hidden";
const ID_X_NAME = "x-name";
const ID_X_STATUS = "x-status";
const ID_X_PATH = "x-path";
const ID_ENTRY_TEMPLATE = "entry-template";

function init() {
    console.log("init searcher")

    let form = document.getElementById('search-form');

    let search_box = document.getElementById(ID_SEARCH_BOX)
    console.log("for box", search_box)

    // todo: wut why no work on Firefox
    search_box.addEventListener("search", (e) => {
        console.log("search=event", e)
    })
    // instead do
    search_box.addEventListener("keyup", async (e) => {
        console.log("key-event", e)
        await push_search()
    })

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        set_message(search_box.value);
        console.log("submit-event", e)
        await push_search()
    })
}

function set_message(value) {
    console.info(`display message: ${value}`);
    let tmp_output = document.getElementById("message");
    tmp_output.innerText = value;
}

// document.addEventListener("DOMContentLoaded", init)
window.addEventListener("load", init)

// idk
const STATE_OFF = 1;
const STATE_RUNNING = 2;
// const STATE_RUNNING_AND_NEXT = 3;
const search_state = {
    state: STATE_OFF,
    next_query: null,
    last_query: null,
};

async function push_search() {
    let new_search = document.getElementById(ID_SEARCH_BOX).value;
    search_state.next_query = new_search;

    await update_search()
}

async function update_search() {
    if (
        // first call
        !(search_state.next_query == null && search_state.last_query == null) &&
        (
            search_state.next_query == null ||
            search_state.next_query === search_state.last_query
        )
    ) {
        console.debug(`ignore next ${search_state.next_query} last ${search_state.last_query}`)
        search_state.state = STATE_OFF;
        search_state.next_query = null;
        return;
    }

    let debug = debug_search_state();
    let is_fetch;
    if (search_state.state === STATE_OFF) {
        set_message(`fetch search "${search_state.next_query} - ${debug}"`)
        is_fetch = true;
    } else if (search_state.state === STATE_RUNNING) {
        set_message(`queue search "${search_state.next_query} - ${debug}"`)
        is_fetch = false;
    } else {
        throw new Error("unknown")
    }

    if (is_fetch) {
        if (search_state.next_query == null) {
            throw new Error("expected value")
        }
        search_state.state = STATE_RUNNING;

        let cached_next = search_state.next_query;
        let url = `/browse/tor?query=${cached_next}`;
        console.info(`fetch ${url} - ${debug_search_state()}`)
        let response;
        try {
            let response_raw = await fetch(url)
            response = await response_raw.json()
            search_state.state = STATE_OFF
        } catch (e) {
            console.error(`failed fetch ${url} - ${e}`)
        }
        set_search_results(response)
        if (search_state.next_query === cached_next) {
            search_state.next_query = null
        }
        search_state.last_query = cached_next;
        await update_search()
    }
}

function debug_search_state() {
    let state;
    if (search_state.state === STATE_OFF) {
        state = "OFF"
    } else if (search_state.state === STATE_RUNNING) {
        state = "RUNNING"
    } else {
        return "SearchState ??UNKNOWN??"
    }
    return `SearchState ${state}`
}

function set_search_results(tor_entries) {
    set_message(`found ${tor_entries.length} - ${debug_search_state()}`)

    let template = document.querySelector(`#${ID_ENTRY_TEMPLATE}`);

    tor_entries.reverse();
    for (const existing_root of document.querySelectorAll(`.${ID_X_ENTRY}`)) {
        if (existing_root.id === ID_ENTRY_TEMPLATE) {
            continue;
        }

        let next = tor_entries.pop();
        if (next === undefined) {
            // list is shorter than the existing entries
            existing_root.classList.add(ID_X_ENTRY_HIDDEN)
        } else {
            existing_root.classList.remove(ID_X_ENTRY_HIDDEN)
            set_search_result(existing_root, next)
        }
    }
    while (tor_entries.length !== 0) {
        let next = tor_entries.pop();

        let new_entry = template.cloneNode(true);
        new_entry.id = "";
        new_entry.classList.remove(ID_X_ENTRY_HIDDEN)
        set_search_result(new_entry, next);
        template.parentElement.appendChild(new_entry)
    }
}

function set_search_result(root, tor_entry) {
    root.querySelector(`.${ID_X_NAME}`).innerText = tor_entry.name;
    root.querySelector(`.${ID_X_STATUS}`).innerText = tor_entry.status;
    root.querySelector(`.${ID_X_PATH}`).innerText = tor_entry.path
}