use botshop_v2::util::db::{insert_user, query_user, update_user, User};
use chrono::Utc;
use clap::{ArgEnum, Parser, Subcommand};
#[derive(Parser)]
#[clap(name = "DB Util")]
#[clap(author = "Xetera Mnemonics <grostaco@gmail.com>")]
#[clap(version = "0.1")]
#[clap(about = "Manage the user record database")]
pub struct Cli {
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
        /// Record's name to be inserted
        name: String,
        /// Record's points to be inserted
        points: u8,
        /// Record's timestamp to be inserted. Blank if it's not completed.
        timestamp: Option<i64>,
    },

    /// Delete a record from a record type
    Delete {
        /// Index of the record to delete
        index: usize,
    },

    /// Update an existing record in a record type
    Update {
        /// Index of the record to update
        index: usize,
        /// Record's new name
        name: String,
        /// Record's new points
        points: u8,
        /// Record's new timestamp. Blank if it's not completed.
        timestamp: Option<i64>,
    },
}

#[derive(Copy, Clone, ArgEnum)]
enum RecordType {
    Daily,
    Pending,
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
                RecordType::Pending => &mut user.periodic,
                RecordType::Transaction => &mut user.transactions,
            };

            match commands {
                ModifySub::Update {
                    index,
                    name,
                    points,
                    timestamp,
                } => {
                    let mut timestamp = *timestamp;
                    if matches!(record_type, RecordType::Transaction) && timestamp.is_none() {
                        timestamp = Some(Utc::now().timestamp())
                    }
                    let index = *index;
                    if index >= record.0.len() {
                        record.0.push((name.to_string(), *points, timestamp));
                    } else {
                        record.0[index] = (name.to_string(), *points, timestamp);
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
                    let mut timestamp = *timestamp;
                    if matches!(record_type, RecordType::Transaction) && timestamp.is_none() {
                        timestamp = Some(Utc::now().timestamp())
                    }

                    record.0.push((name.to_string(), *points, timestamp));
                }
            }
            update_user(&cli.dbfile, &user).expect("Cannot update for user");
        }
    }
}
