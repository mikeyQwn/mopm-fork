use storage::storage::Storage;

mod core;
mod storage;

fn main() {
    Storage::init().unwrap();

    let _ = Storage::clear();
}
