use clap::Parser;
use parquet::{arrow::arrow_reader::ParquetRecordBatchReaderBuilder, file::reader::{FileReader, SerializedFileReader}};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// A list of file names to be interpolated. List should be delimited by spaces and is processed in order provided 
	#[arg(long, required = true, num_args = 1..)]
	file: Vec<String>,
}

fn main() {
	let args = Args::parse();
	for file in args.file {
		let file = File::open(file);
		let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
		let mut reader = builder.build();
		while let Some(batch) = reader.next() {
			let batch: RecordBatch = batch;
			println!("Read batch with {} rows", batch.num_rows());
		}
	}
}