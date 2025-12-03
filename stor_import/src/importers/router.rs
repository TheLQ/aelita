use aelita_stor_diesel::StorTransaction;

pub trait Importer {
    fn process(conn: &mut StorTransaction, raw_data: Vec<u8>);
}
