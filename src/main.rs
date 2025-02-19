use std::{collections::HashMap, env, fs::{self, File}, io::Write, path::Path, process};
use serde::Deserialize;
use clap::{Parser, Subcommand};

const DEFAULT_TOML_FILE: &str = "/usr/local/etc/q/q.toml";
const DEFAULT_TOML_CONTENT: &str = r#"group = [
{ name = "show files", command = "ls -lh", help = "this is help info" }
]"#;

#[derive(Subcommand, Clone, Debug)]
pub enum SecondCommandGroup {
    #[command(about="create default toml file.")]
    Init,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from(DEFAULT_TOML_FILE))]
    config: String,

    #[command(subcommand)]
    second_command: Option<SecondCommandGroup>
}

fn create_default_toml_file() -> anyhow::Result<()> {
    let toml_path = Path::new(DEFAULT_TOML_FILE);
    if !toml_path.exists() {
        let prefix = toml_path.parent().unwrap();
        fs::create_dir_all(prefix)?;
        let mut file = File::create(DEFAULT_TOML_FILE)?;
        file.write_all(DEFAULT_TOML_CONTENT.to_string().as_bytes())?;
        println!("create {} ok", DEFAULT_TOML_FILE);
    } else {
        println!("{} is existed!", DEFAULT_TOML_FILE);
    }
    Ok(())
}

#[derive(Deserialize, Debug)]
struct CommandMap {
    group: Vec<HashMap<String, String>>
}

fn do_alias_command() -> anyhow::Result<()> {
    let content = fs::read_to_string(DEFAULT_TOML_FILE)?;
    let config: CommandMap = toml::from_str(&content)?;
    let sub_command = env::args().into_iter().skip(1).collect::<Vec<String>>().join(" ");
    for one_command in config.group {
        if let Some(name) = one_command.get("name") {
            if *name == sub_command {
                let real_command = one_command.get("command").unwrap();
                let output = process::Command::new("sh")
                    .arg("-c")
                    .arg(real_command)
                    .output()
                    .expect("failed to execute process");
                if output.status.success() {
                    println!("{}", String::from_utf8(output.stdout)?);
                } else {
                    println!("{}", String::from_utf8(output.stderr)?);
                }

                return Ok(())
            }
        }
    }

    anyhow::bail!("not found command")
}

fn print_help_info() -> anyhow::Result<()> {
    let content = fs::read_to_string(DEFAULT_TOML_FILE)?;
    let config: CommandMap = toml::from_str(&content)?;
    let help_text = config.group.iter().map(|one_command| -> String {
        format!("{} \"{}\"", one_command.get("name").unwrap(), one_command.get("help").unwrap())
    }).collect::<Vec<String>>().join("\n");
    println!("{}", help_text);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    match Args::try_parse() {
        Ok(args) => {
            match args.second_command {
                Some(second_command_group) => {
                    match second_command_group {
                        SecondCommandGroup::Init => {
                            create_default_toml_file()?
                        }
                    }
                },
                None => {
                    print_help_info()?;
                }
            }
        },
        Err(err) => {
            let second_arg = env::args().skip(1).collect::<Vec<_>>().join("");
            if second_arg == "help" {
                print!("{}", err);
            } else {
                do_alias_command()?;
            }
        }
    }

    Ok(())
}
