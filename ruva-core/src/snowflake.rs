//! This is to generate global identifier

use std::hint::spin_loop;
use std::sync::atomic::{AtomicI16, AtomicI64, Ordering};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct NumericalUniqueIdGenerator {
	/// epoch used by the snowflake algorithm.
	epoch: SystemTime,

	/// datacenter_id and machine_id are fixed once the system is up running.
	/// Any changes in datacenter IDs require careful review since an accidental change in those values can lead to ID conflicts
	/// make sure that none of them is bigger than 5bits
	pub datacenter_id: i32,
	pub machine_id: i32,

	/// Most important 41 bits make up the timestamp section. As timestamps grow with time, IDs are sortable by time.
	/// Maximum timestamp that can be represented in 41 bits is 2^41 -1 = 2199023255551 give is around 69 years.
	timestamp: AtomicI64,

	/// Sequence number is 12 bits, give gives us 2^12 combinations. This field is 0 unless more than one ID is generated in a millisecond on the same server
	sequence_num: AtomicI16,
}

#[derive(Debug)]
pub struct NumericalUniqueIdBucket {
	/// Hidden the `NumericalUniqueIdGenerator` in bucket .
	snowflake_id_generator: NumericalUniqueIdGenerator,

	/// The bucket buffer;
	bucket: Vec<i64>,
}

impl NumericalUniqueIdGenerator {
	/// Constructs a new `NumericalUniqueIdGenerator` using the UNIX epoch.
	///
	/// # Examples
	///
	/// ```
	/// use snowflake::NumericalUniqueIdGenerator;
	///
	/// let id_generator = NumericalUniqueIdGenerator::new(1, 1);
	/// ```
	pub fn new(datacenter_id: i32, machine_id: i32) -> NumericalUniqueIdGenerator {
		Self::with_epoch(datacenter_id, machine_id, UNIX_EPOCH)
	}

	/// Constructs a new `NumericalUniqueIdGenerator` using the specified epoch.
	///
	/// # Examples
	///
	/// ```
	/// use std::time::{Duration, UNIX_EPOCH};
	/// use snowflake::NumericalUniqueIdGenerator;
	///
	/// // 1 January 2015 00:00:00
	/// let discord_epoch = UNIX_EPOCH + Duration::from_millis(1420070400000);
	/// let id_generator = NumericalUniqueIdGenerator::with_epoch(1, 1, discord_epoch);
	/// ```
	pub fn with_epoch(datacenter_id: i32, machine_id: i32, epoch: SystemTime) -> NumericalUniqueIdGenerator {
		//TODO:limit the maximum of input args datacenter_id and machine_id
		let timestamp = current_time_in_milli(epoch);

		NumericalUniqueIdGenerator {
			epoch,
			timestamp: AtomicI64::new(timestamp),
			datacenter_id,
			machine_id,
			sequence_num: AtomicI16::new(0),
		}
	}

	/// within 64 bits:
	/// sign bit and timestamp takes 42 bits so, left shift 22
	/// datacenter id takes 5 bits in the second place so left shift 17
	/// machine id takes 5 bits in the third place so left shift 12
	/// sequence number comes last.
	fn get_snowflake(&self) -> i64 {
		self.timestamp.load(Ordering::Relaxed) << 22 | ((self.datacenter_id << 17) as i64) | ((self.machine_id << 12) as i64) | (self.sequence_num.load(Ordering::Relaxed) as i64)
	}

	/// The basic guarantee time punctuality.
	///
	/// Basic guarantee time punctuality.

	/// When traffic peaks, 4096 in a millsec is simply not enough.
	/// But setting time after every 4096 calls.
	///
	/// # Examples
	///
	/// ```
	/// use snowflake::NumericalUniqueIdGenerator;
	///
	/// let mut id_generator = NumericalUniqueIdGenerator::new(1, 1);
	/// id_generator.generate();
	/// ```
	pub fn generate(&self) -> i64 {
		self.sequence_num.store((self.sequence_num.load(Ordering::Relaxed) + 1) % 4096, Ordering::Relaxed);

		// Maintenance `timestamp` for every 4096 ids generated.
		if self.sequence_num.load(Ordering::Relaxed) == 0 {
			let mut now_millis = current_time_in_milli(self.epoch);

			// If the following is true, then check if sequence has been created 4092 times,
			// and then busy wait until the next millisecond
			// to prevent 'clock is moving backwards' situation.
			if now_millis == self.timestamp.load(Ordering::Relaxed) {
				now_millis = race_next_milli(self.timestamp.load(Ordering::Relaxed), self.epoch);
			}

			self.timestamp.store(now_millis, Ordering::Relaxed);
		}

		self.get_snowflake()
	}
}

// TODO Get the following concept
impl NumericalUniqueIdBucket {
	/// Constructs a new `NumericalUniqueIdBucket` using the UNIX epoch.
	/// Please make sure that datacenter_id and machine_id is small than 32(2^5);
	///
	/// # Examples
	///
	/// ```
	/// use snowflake::NumericalUniqueIdBucket;
	///
	/// let id_generator_bucket = NumericalUniqueIdBucket::new(1, 1);
	/// ```
	pub fn new(datacenter_id: i32, machine_id: i32) -> Self {
		Self::with_epoch(datacenter_id, machine_id, UNIX_EPOCH)
	}

	/// Constructs a new `NumericalUniqueIdBucket` using the specified epoch.
	/// Please make sure that datacenter_id and machine_id is small than 32(2^5);
	///
	/// # Examples
	///
	/// ```
	/// use std::time::{Duration, UNIX_EPOCH};
	/// use snowflake::NumericalUniqueIdBucket;
	///
	/// // 1 January 2015 00:00:00
	/// let beringlab = UNIX_EPOCH + Duration::from_millis(1570292856000);
	/// let id_generator_bucket = NumericalUniqueIdBucket::with_epoch(1, 1, beringlab);
	/// ```
	pub fn with_epoch(datacenter_id: i32, machine_id: i32, epoch: SystemTime) -> Self {
		let snowflake_id_generator = NumericalUniqueIdGenerator::with_epoch(datacenter_id, machine_id, epoch);
		let bucket = Vec::new();

		NumericalUniqueIdBucket { snowflake_id_generator, bucket }
	}

	/// # Examples
	///
	/// ```
	/// use snowflake::NumericalUniqueIdBucket;
	///
	/// let mut id_generator_bucket = NumericalUniqueIdBucket::new(1, 1);
	/// let id = id_generator_bucket.get_id();
	///
	/// ```
	pub fn get_id(&mut self) -> i64 {
		// 247 ns/iter
		// after self.bucket.push(self.snowflake_id_generator.generate());

		if self.bucket.is_empty() {
			self.fill_bucket();
		}
		self.bucket.pop().unwrap()
	}

	fn fill_bucket(&mut self) {
		// 1,107,103 -- 1,035,018 ns/iter
		//self.bucket.push(self.snowflake_id_generator.generate());

		for _ in 0..4091 {
			self.bucket.push(self.snowflake_id_generator.generate());
		}
	}
}

#[inline(always)]
/// Get the latest milliseconds of the clock.
pub fn current_time_in_milli(epoch: SystemTime) -> i64 {
	SystemTime::now().duration_since(epoch).expect("System Time Error!").as_millis() as i64
}

#[inline(always)]
// Constantly refreshing the latest milliseconds by busy waiting.
fn race_next_milli(timestamp: i64, epoch: SystemTime) -> i64 {
	let mut latest_time_millis: i64;
	loop {
		latest_time_millis = current_time_in_milli(epoch);
		if latest_time_millis > timestamp {
			return latest_time_millis;
		}
		spin_loop();
	}
}

pub fn id_generator() -> &'static NumericalUniqueIdGenerator {
	use std::sync::OnceLock;
	static SNOWFLAKE: OnceLock<NumericalUniqueIdGenerator> = OnceLock::new();
	SNOWFLAKE.get_or_init(|| {
		NumericalUniqueIdGenerator::new(
			std::env::var("DATACENTER_ID").expect("DATACENTER_ID MUST BE SET").parse::<i32>().expect("Parsing Failed!"),
			std::env::var("MACHINE_ID").expect("MACHINE_ID MUST BE SET").parse::<i32>().expect("Parsing Failed!"),
		)
	})
}

#[test]
fn test_generate() {
	let id_generator = NumericalUniqueIdGenerator::new(1, 2);
	let mut ids = Vec::with_capacity(10000);

	for _ in 0..99 {
		for _ in 0..10000 {
			ids.push(id_generator.generate());
		}

		ids.sort();
		ids.dedup();

		assert_eq!(10000, ids.len());

		ids.clear();
	}
}

#[test]
fn test_singleton_generate() {
	let id_generator = id_generator();
	let mut ids = Vec::with_capacity(1000000);

	for _ in 0..99 {
		for _ in 0..1000000 {
			ids.push(id_generator.generate());
		}

		assert_eq!(1000000, ids.len());
		assert!(ids.first().unwrap() < ids.last().unwrap());
		assert!(ids.get(999998).unwrap() < ids.get(999999).unwrap());

		ids.clear();
	}
}
