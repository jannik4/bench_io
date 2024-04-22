use anyhow::{Context, Result};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    // Parse args
    let cmd = Cmd::from_env().context("failed to parse args")?;

    // Run
    match cmd.sub {
        SubCmd::Write {
            file,
            block_size,
            count,
        } => {
            let mut time = Duration::ZERO;
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&file)
                .with_context(|| format!("Failed to open {} for writing.", file))?;

            let buf = vec![0; block_size as usize];

            for _ in 0..count {
                let start = Instant::now();
                file.write_all(&buf)?;
                time += start.elapsed();
            }

            println!("Finished in {:.6}s", time.as_secs_f64());
        }
        SubCmd::Read {
            file,
            block_size,
            count,
        } => {
            let mut time = Duration::ZERO;
            let mut file = OpenOptions::new()
                .read(true)
                .open(&file)
                .with_context(|| format!("Failed to open {} for reading.", file))?;

            let mut buf = vec![0; block_size as usize];

            for _ in 0..count {
                let start = Instant::now();
                file.read_exact(&mut buf)?;
                time += start.elapsed();
            }

            println!("Finished in {:.6}s", time.as_secs_f64());
        }
        SubCmd::Test {
            file_write,
            file_read,
            block_size,
            count,
        } => {
            let mut file_write = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&file_write)
                .with_context(|| format!("Failed to open {} for writing.", file_write))?;
            let mut file_read = OpenOptions::new()
                .read(true)
                .open(&file_read)
                .with_context(|| format!("Failed to open {} for reading.", file_read))?;

            // Write
            let buf_write = (0..block_size * count)
                .map(|v| (v % 256) as u8)
                .collect::<Vec<_>>();
            for i in 0..count {
                file_write.write_all(
                    &buf_write[(i * block_size) as usize..((i + 1) * block_size) as usize],
                )?;
            }

            // Read
            let mut buf_read = vec![0; (block_size * count) as usize];
            for i in 0..count {
                file_read.read_exact(
                    &mut buf_read[(i * block_size) as usize..((i + 1) * block_size) as usize],
                )?;
            }

            // Compare
            if buf_write == buf_read {
                println!("Test passed");
            } else {
                return Err(anyhow::anyhow!("Test failed"));
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Cmd {
    sub: SubCmd,
}

#[derive(Debug)]
enum SubCmd {
    Write {
        file: String,
        block_size: u64,
        count: u64,
    },
    Read {
        file: String,
        block_size: u64,
        count: u64,
    },
    Test {
        file_write: String,
        file_read: String,
        block_size: u64,
        count: u64,
    },
}

impl Cmd {
    fn from_env() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        let sub = match args.subcommand()?.as_deref() {
            Some("write") => SubCmd::Write {
                file: args.value_from_str(["-f", "--file"])?,
                block_size: args
                    .opt_value_from_str(["-s", "--block-size"])?
                    .unwrap_or(32),
                count: args.opt_value_from_str(["-c", "--count"])?.unwrap_or(1),
            },
            Some("read") => SubCmd::Read {
                file: args.value_from_str(["-f", "--file"])?,
                block_size: args
                    .opt_value_from_str(["-s", "--block-size"])?
                    .unwrap_or(32),
                count: args.opt_value_from_str(["-c", "--count"])?.unwrap_or(1),
            },
            Some("test") => {
                let file = args.opt_value_from_str(["-f", "--file"])?;
                let file_write = args
                    .opt_value_from_str("--file-write")?
                    .or_else(|| file.clone())
                    .context("missing --file-write")?;
                let file_read = args
                    .opt_value_from_str("--file-read")?
                    .or(file)
                    .context("missing --file-read")?;

                SubCmd::Test {
                    file_write,
                    file_read,
                    block_size: args
                        .opt_value_from_str(["-s", "--block-size"])?
                        .unwrap_or(32),
                    count: args.opt_value_from_str(["-c", "--count"])?.unwrap_or(1),
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid subcommand")),
        };

        Ok(Self { sub })
    }
}
