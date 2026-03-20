use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = pylob::stub_info()?;
    stub.generate()?;
    Ok(())
}
