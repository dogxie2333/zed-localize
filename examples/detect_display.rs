fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let file = std::fs::File::open(&path)?;
        let bytes = unsafe { memmap2::Mmap::map(&file)? };
        let start = std::time::Instant::now();
        let caller = zed_zh_addon::display_adapter::detect(&bytes)?;
        println!(
            "{}",
            serde_json::json!({"path":path,"caller":caller.map(|rva|format!("0x{rva:x}")),"milliseconds":start.elapsed().as_secs_f64()*1000.0,"version_gate":false})
        );
    }
    Ok(())
}
