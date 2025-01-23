use aelita_stor_diesel::connection::establish_connection;
use aelita_stor_diesel::models::xrn_registry::XrnExtraction;
use aelita_stor_diesel::models::*;
use aelita_stor_diesel::schema::xrn_registry::dsl::xrn_registry;
use diesel::prelude::*;

fn main() {
    let connection = &mut establish_connection();

    let new_post = NewXrnExtraction {
        xrn: "asdf".into(),
        published: "asdf".into(),
    };

    diesel::insert_into(xrn_registry)
        .values(&new_post)
        .execute(connection)
        .expect("Error saving new post");
    println!("inserted post");

    let results = xrn_registry
        .select(XrnExtraction::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.xrn);
        println!("-----------\n");
        println!("{}", post.published);
    }
}
