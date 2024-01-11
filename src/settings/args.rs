use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory of searching root
    #[clap(short = 'i')]
    #[arg(long)]
    pub input_path: String,

    /// Output directory of checking results
    #[clap(short = 'o')]
    #[arg(long)]
    pub output_path: String,

    /// Number of threads used to calculate
    #[clap(short = 't')]
    #[arg(long)]
    pub threads: Option<usize>,
}
