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
    _ => unreachable!(),
  };
}

fn get(cmd: GetCommand) -> Result<()> {
  let current_dir = std::env::current_dir()?;
  let mut store = kvs::KvStore::open(current_dir)?;
  let res = store.get(cmd.key)?;

  match res {
    Some(val) => println!("{}", val),
    None => println!("Key not found"),
  };

  Ok(())
}

fn set(cmd: SetCommand) -> Result<()> {
  let current_dir = std::env::current_dir()?;
  let mut store = kvs::KvStore::open(current_dir)?;
  store.set(cmd.key, cmd.value)?;
  Ok(())
}

fn rm(cmd: RemoveCommand) -> Result<()> {
  let current_dir = std::env::current_dir()?;
  let mut store = kvs::KvStore::open(current_dir)?;
  store.remove(cmd.key)?;
  Ok(())
}
