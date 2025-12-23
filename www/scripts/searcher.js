"use strict";

const ID_SEARCH_BOX = "search-box";
const ID_X_ENTRY = "x-entry";
const ID_X_ENTRY_HIDDEN = "x-entry-hidden";
const ID_ENTRY_TEMPLATE = "entry-template";
const ID_DISPLAY_TEMPLATE = "display-template";

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

    // browser might cache last search term
    push_search()
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
    next_query: "",
    last_query: "",
};

async function push_search() {
    let new_search = document.getElementById(ID_SEARCH_BOX).value;
    search_state.next_query = new_search;

    await update_search()
}

async function update_search() {
    if (
        search_state.next_query === "" ||
        search_state.next_query === search_state.last_query
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

    let entry_template = document.querySelector(`#${ID_ENTRY_TEMPLATE}`);
    let display_template = document.querySelector(`#${ID_DISPLAY_TEMPLATE}`);

    tor_entries.reverse();
    for (const existing_root of document.querySelectorAll(`.${ID_X_ENTRY}`)) {
        if (existing_root.id === ID_ENTRY_TEMPLATE) {
            continue;
        }

        let next = tor_entries.pop();
        if (next === undefined) {
            // list is shorter than the existing entries
            existing_root.classList.add(ID_X_ENTRY_HIDDEN)
            existing_root.nextElementSibling.classList.add(ID_X_ENTRY_HIDDEN)
        } else {
            if (existing_root.classList != null) {
                existing_root.classList.remove(ID_X_ENTRY_HIDDEN)
            }
            existing_root.nextElementSibling.classList.remove(ID_X_ENTRY_HIDDEN)
            set_search_result([existing_root], next)
        }
    }
    while (tor_entries.length !== 0) {
        let next = tor_entries.pop();

        let new_entry = entry_template.cloneNode(true);
        new_entry.id = "";
        new_entry.classList.remove(ID_X_ENTRY_HIDDEN)
        // let new_display = display_template.cloneNode(true);
        // new_display.id = "";
        // new_display.classList.remove(ID_X_ENTRY_HIDDEN)

        entry_template.parentElement.appendChild(new_entry)
        // display_template.parentElement.appendChild(new_display)

        set_search_result([
            new_entry,
            //new_display
        ], next);

    }
}

function set_search_result(roots, tor_entry) {
    for (const clazz of Object.keys(tor_metas)) {
        let json_field = tor_metas[clazz];
        let json_value = tor_entry[json_field];

        if (json_field === "progress") {
            json_value = (parseFloat(json_value) * 100).toFixed(0);
            // json_value = `%${json_value}`;
        }
        console.log(`applying ${json_field}=${json_value} class ${clazz}`, tor_entry);

        let not_found = true;
        for (const root of roots) {
            let elem = root.querySelector(`.${clazz}`);
            if (elem === null) {
                console.debug(`class ${clazz} not found for`, root)
                continue
            } else {
                console.debug(`class ${clazz} found`, root)
            }
            elem.innerText = json_value;
            not_found = false;
            break;
        }
        if (not_found) {
            throw new Error(`no elem found class ${clazz} field ${json_field}`)
        }
    }
}

let tor_metas = {
    "s-name": "name",
    "s-state": "state",
    "s-path": "path",
    "s-progress": "progress",
    "s-added": "added_on",
    "s-completed": "completion_on",
    "s-size": "original_size",
    "s-downloaded": "downloaded",
    "s-uploaded": "uploaded",
    "s-time-active": "secs_active",
    "s-time-seeding": "secs_seeding",
}