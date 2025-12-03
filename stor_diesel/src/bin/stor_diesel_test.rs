use aelita_stor_diesel::tests::todo_list::create_todo_list;
use aelita_stor_diesel::{PermaStore, establish_connection, log_init_trace};
use xana_commons_rs::pretty_format_error;

fn main() {
    log_init_trace();

    let conn = &mut establish_connection(PermaStore::AelitaNull).unwrap();

    let res = match 1 {
        1 => create_todo_list(conn),
        i => {
            panic!("unhandled number {}", i)
        }
    };

    if let Err(e) = res {
        panic!("MainFail {}", pretty_format_error(&e))
    }
}
