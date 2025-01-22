use chrono::NaiveDateTime;
use stor_diesel::connection::establish_connection;
use stor_diesel::models::*;
use diesel::prelude::*;
use stor_diesel::models::xrn_registry::{NewPost, XrnExtraction};
use stor_diesel::schema::xrn_registry::dsl::xrn_registry;

fn main() {
    let connection = &mut establish_connection();

    let new_post = NewPost { xrn: "asdf", published: NaiveDateTime::default() };

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