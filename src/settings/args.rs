use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory of searching root
    #[arg(short, long)]
    pub input_path: String,

    /// Output directory of checking results
    #[arg(short, long)]
    pub output_path: String,

    /// Number of threads used to calculate
    #[arg(short, long)]
    pub threads: Option<usize>,
}
