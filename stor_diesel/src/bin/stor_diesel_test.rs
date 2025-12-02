use aelita_commons::err_utils::pretty_error;
use aelita_stor_diesel::common::log_init_trace;
use aelita_stor_diesel::connection::{PermaStore, establish_connection};
use aelita_stor_diesel::tests::todo_list::create_todo_list;

fn main() {
    log_init_trace();

    let conn = &mut establish_connection(PermaStore::AelitaNull);

    let res = match 1 {
        1 => create_todo_list(conn),
        i => {
            panic!("unhandled number {}", i)
        }
    };

    if let Err(e) = res {
        panic!("MainFail {}", pretty_error(e))
    }
}
