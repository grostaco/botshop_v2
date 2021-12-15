use std::fs::File;

/// A struct to represent every daily tasks and corresponding files
pub struct Daily {
    /// The file to load daily tasks from
    source_file: File,

    transaction_file: File,
    records: Vec<(String, u8, Option<i64>)>,
}

impl Daily {
    fn new(source_file: &str, transaction_file: &str) -> Self {
        let source = File::open(source_file).expect("Unable to open source file");

        let mut rdr = csv::Reader::from_reader(source);
        let mut records = Vec::new();
        for record in rdr.records() {
            let record = record.expect("Record cannot be read");
            records.push((
                record.get(0).expect("Expected task name").to_owned(),
                record
                    .get(1)
                    .expect("Expected points")
                    .parse::<u8>()
                    .expect("Expected points to be integral"),
                match record.get(2).expect("Expected completed") {
                    "None" => None,
                    timestamp => Some(
                        timestamp
                            .parse::<i64>()
                            .expect("Expected timestamp as an integer"),
                    ),
                },
            ));
        }

        Self {
            source_file: File::open(source_file).expect("Unable to open source file"),
            transaction_file: File::open(transaction_file).expect("Unable to transaction file"),
            records,
        }
    }

    fn complete_task() {}
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::Builder;

    #[test]
    fn create_daily() {
        let mut source_file = Builder::new()
            .prefix("source_file")
            .suffix(".csv")
            .tempfile()
            .expect("Unable to create tempfile");

        let transaction_file = Builder::new()
            .prefix("transaction_file")
            .suffix(".csv")
            .tempfile()
            .expect("Unable to create tempfile");

        source_file
            .write(b"task,points,completed\ntask1,8,None\ntask2,8,3222")
            .expect("Unable to write source file");

        let daily = Daily::new(
            source_file.path().to_str().unwrap(),
            transaction_file.path().to_str().unwrap(),
        );

        assert_eq!(
            daily.records,
            vec![
                ("task1".to_owned(), 8, None),
                ("task2".to_owned(), 8, Some(3222))
            ]
        );
    }
}
