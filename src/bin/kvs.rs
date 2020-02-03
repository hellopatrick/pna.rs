use anyhow::Result;
use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// KVStore
struct Args {
  /// version
  #[argh(switch, short = 'V')]
  version: bool,

  #[argh(subcommand)]
  command: Option<Commands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
  Get(GetCommand),
  Set(SetCommand),
  Rm(RemoveCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Get
#[argh(subcommand, name = "get")]
struct GetCommand {
  #[argh(positional)]
  /// how many x
  key: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Get
#[argh(subcommand, name = "set")]
struct SetCommand {
  #[argh(positional)]
  /// how many x
  key: String,
  #[argh(positional)]
  /// how many x
  value: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Get
#[argh(subcommand, name = "rm")]
struct RemoveCommand {
  #[argh(positional)]
  /// how many x
  key: String,
}

fn main() -> Result<()> {
  let args: Args = argh::from_env();

  if args.version {
    println!(env!("CARGO_PKG_VERSION"));
    return Ok(());
  }

  return match args.command {
    Some(Commands::Get(arg)) => get(arg),
    Some(Commands::Set(arg)) => set(arg),
    Some(Commands::Rm(arg)) => rm(arg),
    _ => todo!(),
  };
}

fn get(_: GetCommand) -> Result<()> {
  eprintln!("unimplemented");
  todo!();
}

fn set(_: SetCommand) -> Result<()> {
  eprintln!("unimplemented");
  todo!();
}

fn rm(_: RemoveCommand) -> Result<()> {
  eprintln!("unimplemented");
  todo!();
}
