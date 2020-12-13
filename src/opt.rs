use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    pub remap: PathBuf,
    #[structopt(short = "u", long = "input-user-mappings")]
    pub user_mapping: Option<PathBuf>,
}
