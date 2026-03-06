fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Scylla 1.x types are large; the default 1MB stack on Windows is insufficient
    // for the async state machine in Database::new(). Run on a thread with 8MB stack.
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
    let handler = builder
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(voxov::run())
        })
        .unwrap();
    handler.join().unwrap()
}
