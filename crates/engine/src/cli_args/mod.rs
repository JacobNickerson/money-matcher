use crate::simulator::latency_config::{JitterKind, SimJitter};
use clap::{Parser, Subcommand};
use rand_distr::{Normal, Uniform};

pub fn prob_parser(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| "invalid float")?;
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err("must be between 0 and 1".into())
    }
}

pub fn positive_float_parser(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| "invalid float")?;
    if val > 0.0 {
        Ok(val)
    } else {
        Err("must be > 0.0".into())
    }
}

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Type of source for generation of synthetic events
    #[command(subcommand)]
    pub event_source: EventSourceType,

    /// Simulated latency in nanoseconds
    #[arg(long, default_value_t = 0)]
    pub sim_latency: u64,

    /// Simulated jitter sampling type
    #[arg(long, default_value = "none")]
    pub sim_jitter_type: JitterKind,

    /// Simulated jitter uniform distribution lower bound
    #[arg(long, required_if_eq("sim_jitter_type", "uniform"))]
    pub low: Option<u64>,
    /// Simulated jitter uniform distribution upper bound
    #[arg(long, required_if_eq("sim_jitter_type", "uniform"))]
    pub high: Option<u64>,

    /// Simulated jitter normal distribution mean
    #[arg(long, required_if_eq("sim_jitter_type","normal"), value_parser = positive_float_parser)]
    pub mean: Option<f64>,
    /// Simulated jitter normal distribution standard deviation
    #[arg(long, required_if_eq("sim_jitter_type","normal"), value_parser = positive_float_parser)]
    pub std_dev: Option<f64>,

    /// RNG seed for randomly sampled values, if unspecified a random one is picked
    #[arg(long)]
    pub seed: Option<u64>,

    /// Attempt to run the simulation in real-time by attempting to keep sim time and real time synchronized
    #[arg(long)]
    pub real_time: bool,

    /// Log errors and initialization steps to stdout
    #[arg(long, default_value_t = false)]
    pub logging: bool,

    /// Records runtime and events processed and outputs to stdout after simulator finishes generating orders
    ///
    /// Output is in CSV format: step_count,run_time(nanosec),sim_time(nanosec)
    #[arg(long, default_value_t = false)]
    pub benchmark: bool,
}

#[derive(Debug, Subcommand, Clone)]
pub enum EventSourceType {
    /// Generate synthetic orders by randomly sampling inter-arrival times from an exponential distribution
    Poisson {
        /// Number of orders to generate before terminating, if unused the simulation runs indefinitely
        #[arg(long)]
        count: Option<u64>,

        /// Rate of production of synthetic orders in orders per second
        #[arg(long, default_value_t = 100_000.0, value_parser = positive_float_parser)]
        order_rate: f64,

        /// Proportion of synthetic orders that are bids vs asks, value must be between 0-1
        #[arg(long, default_value_t = 0.5, value_parser = prob_parser)]
        bid_rate: f64,

        /// Proportion of synthetic orders that are new limits, value must be between 0-1 and must sum to 1 with the other event types
        #[arg(long, default_value_t = 0.5, value_parser = prob_parser)]
        new_limit_rate: f64,

        /// Proportion of synthetic orders that are cancels, value must be between 0-1 and must sum to 1 with the other event types
        #[arg(long, default_value_t = 0.4, value_parser = prob_parser)]
        cancel_rate: f64,

        /// Proportion of synthetic orders that are market orders, value must be between 0-1 and must sum to 1 with the other event types
        #[arg(long, default_value_t = 0.05, value_parser = prob_parser)]
        market_rate: f64,

        /// Proportion of synthetic orders that are updates, value must be between 0-1 and must sum to 1 with the other event types
        #[arg(long, default_value_t = 0.05, value_parser = prob_parser)]
        update_rate: f64,

        /// Average order price in cents, must be a positive, non-zero value
        #[arg(long, default_value_t = 1000.0, value_parser = positive_float_parser)]
        avg_price: f64,

        /// Standard deviation of order price in cents, must be a positive, non-zero value
        #[arg(long, default_value_t = 50.0, value_parser = positive_float_parser)]
        price_dev: f64,
    },
    /// Replay a historical record of order data from a file, file must contain binary data logged using --record
    File {
        /// File path to file containing binary-mapped order data
        #[arg(long, required = true)]
        file_name: String,

        /// Batch size for batch-reading from file
        #[arg(long, default_value_t = 64)]
        batch_size: usize,
    },
}

pub fn validate(args: &Args) -> Result<(), String> {
    match args.sim_jitter_type {
        JitterKind::None => {}
        JitterKind::Normal => {}
        JitterKind::Uniform => {
            if args.low.is_none() {
                return Err("uniform jitter: `low` was not provided".into());
            }
            if args.high.is_none() {
                return Err("uniform jtiter: `high` was not provided".into());
            }
            let low = args.low.unwrap();
            let high = args.high.unwrap();
            if low > high {
                return Err("uniform jitter: `low` must be <= `high`".into());
            }
        }
    }
    Ok(())
}
