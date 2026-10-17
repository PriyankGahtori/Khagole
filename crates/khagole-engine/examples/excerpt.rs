//! Write an excerpt of a JPL kernel: `cargo run --example excerpt -- IN OUT START_JD END_JD`
use khagole_engine::ephemeris::Spk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [input, output, start, end] = args.as_slice() else {
        return Err("usage: excerpt IN OUT START_JD END_JD (TDB Julian dates)".into());
    };
    let bytes = Spk::open(input)?.excerpt(start.parse()?, end.parse()?)?;
    std::fs::write(output, bytes)?;
    Ok(())
}
