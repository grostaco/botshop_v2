use clap::{ArgEnum, Parser, Subcommand};

#[path = "../src/util/db.rs"]
mod db;
use db::{insert_user, query_user, update_user, User};

#[derive(Parser)]
#[clap(name = "DB Util")]
#[clap(author = "Xetera Mnemonics <grostaco@gmail.com>")]
#[clap(version = "0.1")]
#[clap(about = "Manage the user record database")]
struct Cli {
    /// Database file to manage
    #[clap(short, long)]
    dbfile: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query for an existing user
    Query {
        /// User ID to query
        userid: u64,
    },

    /// Add a user
    Add {
        /// User ID to add
        userid: u64,
    },

    /// Modify a user
    Modify {
        /// ID of the user to be modified
        user_id: u64,

        /// Type of record
        #[clap(arg_enum)]
        record_type: RecordType,

        /// Operation type
        #[clap(subcommand)]
        commands: ModifySub,
    },
}

#[derive(Subcommand)]
enum ModifySub {
    /// Insert into a record type
    Insert {
        name: String,
        points: u8,
        timestamp: Option<i64>,
    },

    /// Delete a record from a record type
    Delete { index: usize },

    /// Update an existing record in a record type
    Update {
        index: usize,
        name: String,
        points: u8,
        timestamp: Option<i64>,
    },
}

#[derive(Copy, Clone, ArgEnum)]
enum RecordType {
    Daily,
    Periodic,
    Transaction,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Query { userid } => {
            match query_user(
                &cli.dbfile,
                *userid,
            )
            .expect("Cannot query user")
            {
                Some(user) => println!(
                    "User found!\nID: {}\nDaily Tasks: {:?}\nPeriodic Tasks: {:?}\nTransaction history: {:?}",
                    user.id, user.daily, user.periodic, user.transactions
                ),
                None => println!("Cannot find user with id {}", userid),
            };
        }
        Commands::Add { userid } => {
            insert_user(&cli.dbfile, User::new(*userid)).expect("Cannot add user to the database");
        }

        Commands::Modify {
            user_id,
            record_type,
            commands,
        } => {
            let mut user = query_user(&cli.dbfile, *user_id)
                .expect("Cannot query user")
                .expect("Cannot find specified user under the given ID");

            let record = match record_type {
                RecordType::Daily => &mut user.daily,
                RecordType::Periodic => &mut user.periodic,
                RecordType::Transaction => &mut user.transactions,
            };

            match commands {
                ModifySub::Update {
                    index,
                    name,
                    points,
                    timestamp,
                } => {
                    let index = *index;
                    if index >= record.0.len() {
                        record.push_record(name.to_string(), *points, *timestamp);
                    } else {
                        record.0[index] = (name.to_string(), *points, *timestamp);
                    }
                }
                ModifySub::Delete { index } => {
                    record.0.remove(*index);
                }
                ModifySub::Insert {
                    name,
                    points,
                    timestamp,
                } => {
                    record.push_record(name.to_string(), *points, *timestamp);
                }
            }
            update_user(&cli.dbfile, user).expect("Cannot update for user");
        }
    }
}