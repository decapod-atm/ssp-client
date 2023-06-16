use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use ssp::{Error, Method, Result};
use ssp::jsonrpc::{get_socket_path, Client, JSONRPC_ENV_SOCK, JSONRPC_SOCKET_PATH};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Method to send in the JSON-RPC request
    ///
    /// Example: `enable`
    #[arg(short, long, default_value_t = String::from("status"))]
    method: String,

    /// Run a CLI for interacting with the BAU server.
    #[arg(short='c', long, default_value_t = false)]
    cli: bool,
}

fn print_help() {
    println!("");
    println!("Commands: ");
    println!("\t\tstatus, stat: Gets the current hardware status");
    println!("\t\tenable, e: Enable accepting bills");
    println!("\t\tdisable, d: Disable accepting bills");
    println!("\t\tstack, t: Stack a bill from escrow into storage");
    println!("\t\treject, r: Reject a bill from escrow");
    println!("\t\tquit, exit, q, x: Shutdown the client");
    println!("\t\thelp, h, ?: Print this help dialogue");
    println!("");
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let socket_path = get_socket_path(JSONRPC_ENV_SOCK, JSONRPC_SOCKET_PATH);
    let stop_client_accept = Arc::new(AtomicBool::new(false));

    let mut client = match Client::new(socket_path) {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to setup JSON-RPC client: {err}");
            return Err(err);
        }
    };

    if args.cli {
        let mut rl = DefaultEditor::new().map_err(|err| {
            Error::JsonRpc(format!("failed to setup readline client: {err}"))
        })?;
        let cli_prompt = "json-rpc-client>> ";

        loop {
            let readline = rl.readline(cli_prompt);
            let res = match readline {
                Ok(line) => match line.as_str() {
                    "status" | "stat" => client.send(Method::Status)?,
                    "disable" | "d" => client.send(Method::Disable)?,
                    "enable" | "e" => client.send(Method::Enable)?,
                    "stack" | "t" => client.send(Method::Stack)?,
                    "reject" | "r" => client.send(Method::Reject)?,
                    "reset" => client.send(Method::Reset)?,
                    "quit" | "exit" | "q" | "x" => break,
                    "help" | "h" | "?" => {
                        print_help();
                        Vec::new()
                    }
                    _ => {
                        if line.is_empty() || line.contains("check") || line.contains("res") {
                            if let Ok(responses) = client.receive() {
                                responses
                            } else {
                                Vec::new()
                            }
                        } else {
                            println!("Invalid command");
                            print_help();
                            Vec::new()
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            };

            for response in res {
                println!("{cli_prompt}Server message:\n{response}");
            }
        }

        println!("{cli_prompt}Shutting down");

        stop_client_accept.store(true, Ordering::SeqCst);
    } else {
        let method = Method::from(args.method.as_str());
        let _res = client.send(method)?;
    }

    Ok(())
}
