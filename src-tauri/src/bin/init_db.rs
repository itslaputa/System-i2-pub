fn main() {
    if let Err(error) = system_i2_lib::storage::init_database() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
