use clap::Parser;

#[derive(Parser, Debug)]
pub struct Options {

  #[clap(long)]
  pub since: String,

  #[clap(long)]
  pub new_version: String,


  #[clap(long, default_value=".")]
  pub workspace_path: String,

  #[clap(long)]
  pub replace_all: bool

}
