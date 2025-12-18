"use strict";

const ID_SEARCH_BOX = "search-box";
const ID_X_ENTRY = "x-entry";
const ID_X_ENTRY_HIDDEN = "x-entry-hidden";
const ID_X_NAME = "x-name";
const ID_ENTRY_TEMPLATE = "entry-template";

function init() {
    console.log("init searcher")

    let form = document.getElementById('search-form');

    let search_box = document.getElementById(ID_SEARCH_BOX)
    console.log("for box", search_box)

    // todo: wut why no work on Firefox
    search_box.addEventListener("search", (e) => {
        set_message(e.value);
        console.log("search=event", e)
    })
    // instead do
    search_box.addEventListener("keyup", async (e) => {
        set_message(e.value);
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
    console.info(`search message: ${value}`);
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
};

async function push_search() {
    let new_search = document.getElementById(ID_SEARCH_BOX).value;
    search_state.next_query = new_search;
    console.info(`update search for ${search_state.next_query}`);

    await update_search()
}

async function update_search() {
    let is_fetch;
    if (search_state.state === STATE_OFF) {
        console.log(`fetch search "${search_state.next_query}"`)
        is_fetch = true;
    } else if (search_state.state === STATE_RUNNING) {
        if (search_state.next_query == null) {
            console.log(`queue search "${search_state.next_query}"`)
        } else {
            console.log(`re-queue search "${search_state.next_query}"`)
        }
        is_fetch = false;
    } else {
        throw new Error("unknown")
    }

    if (is_fetch) {
        if (search_state.next_query == null) {
            throw new Error("no value")
        }
        let url = `/browse/tor?prefix=${search_state.next_query}`;
        console.info(`fetch ${url}`)
        try {
            let response_raw = await fetch(url)
            let response = await response_raw.json()
            set_search_results(response)
        } catch (e) {
            console.error(`failed fetch ${url} - ${e}`)
        }
    }
}

function set_search_results(tor_entries) {
    set_message(`found ${tor_entries.length}`)

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
        set_search_result(new_entry, next);
        template.parentElement.appendChild(new_entry)
    }
}

function set_search_result(root, tor_entry) {
    root.querySelector(`.${ID_X_NAME}`).innerText = tor_entry.name
}