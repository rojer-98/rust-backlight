use clap::{AppSettings, Parser, Subcommand};
use cmd_lib::run_cmd;
use std::io::{BufRead, BufReader};
use std::{env, fmt, fs};

const INTEL_BRIGHTNESS_FILE: &'static str = "/sys/class/backlight/intel_backlight/brightness";
const ABSOLUTE_VAL: f32 = 120000f32;

enum CheckErrors {
    RootError,
    IntelFileError,
    WriteError,
    UnknownError,
}
struct BacklightError(CheckErrors);
impl fmt::Debug for BacklightError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            CheckErrors::RootError => {
                write!(f, "Must be root")
            }
            CheckErrors::UnknownError => {
                write!(f, "Something went wrong")
            }
            CheckErrors::WriteError => {
                write!(f, "Can not to write to file {:?}", INTEL_BRIGHTNESS_FILE)
            }
            CheckErrors::IntelFileError => {
                write!(f, "Does not exist {:?}", INTEL_BRIGHTNESS_FILE)
            }
        }
    }
}

fn check_root() -> Result<(), BacklightError> {
    match env::var("USER") {
        Err(_) => Err(BacklightError(CheckErrors::UnknownError)),
        Ok(name) => {
            if name != "root" {
                Err(BacklightError(CheckErrors::RootError))
            } else {
                Ok(())
            }
        }
    }
}

fn operation_with_file() -> Result<Option<u32>, BacklightError> {
    let brightness_file = fs::File::open(INTEL_BRIGHTNESS_FILE)
        .map_err(|_| BacklightError(CheckErrors::IntelFileError))?;
    let mut reader = BufReader::new(&brightness_file);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    let brightness_line = buffer
        .trim()
        .parse::<f32>()
        .map_err(|_| BacklightError(CheckErrors::UnknownError))?;

    let args = Cli::parse();
    match &args.command {
        Commands::Set { digit } => {
            let value: u32 = (ABSOLUTE_VAL * (digit / 100f32)) as u32;
            run_cmd!(sudo bash -c "echo $value >> $INTEL_BRIGHTNESS_FILE")
                .map_err(|_| BacklightError(CheckErrors::WriteError))?;
            println!("Set {}% backlight to file", digit);
            Ok(None)
        }
        Commands::Dec { digit } => {
            let procent: f32 = ((brightness_line * 100f32) / ABSOLUTE_VAL) - digit;
            let value: u32 = (ABSOLUTE_VAL * (procent / 100f32)) as u32;

            run_cmd!(sudo bash -c "echo $value >> $INTEL_BRIGHTNESS_FILE")
                .map_err(|_| BacklightError(CheckErrors::WriteError))?;
            println!("Decrement backlight at procent {}%", digit);
            Ok(None)
        }
        Commands::Inc { digit } => {
            let procent: f32 = ((brightness_line * 100f32) / ABSOLUTE_VAL) + digit;
            let value: u32 = (ABSOLUTE_VAL * (procent / 100f32)) as u32;

            run_cmd!(sudo bash -c "echo $value >> $INTEL_BRIGHTNESS_FILE")
                .map_err(|_| BacklightError(CheckErrors::WriteError))?;
            println!("Increment backlight at procent {}%", digit);
            Ok(None)
        }
        Commands::Get => {
            let procent: u32 = ((brightness_line * 100f32) / ABSOLUTE_VAL) as u32;
            println!("Current backlight: {}%", procent);
            Ok(Some(procent))
        }
    }
}

#[derive(Parser)]
#[clap(name = "rbacklight")]
#[clap(about = "Intel backlight controller")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    #[clap(about = "Set backlight")]
    Set { digit: f32 },
    #[clap(about = "Decrement backlight")]
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Dec { digit: f32 },
    #[clap(about = "Increment backlight")]
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Inc { digit: f32 },
    #[clap(about = "Get current backlight")]
    Get,
}

fn main() {
    check_root()
        .map_err(|x| {
            println!("{:?}", x);
        })
        .unwrap();
    operation_with_file()
        .map_err(|x| {
            println!("{:?}", x);
        })
        .unwrap();
}
