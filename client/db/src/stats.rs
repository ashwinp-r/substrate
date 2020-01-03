use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

/// Accumulated usage statistics for state queries.
pub struct UsageStats {
	started: std::time::Instant,
	reads: AtomicU64,
	bytes_read: AtomicU64,
	writes: AtomicU64,
	bytes_written: AtomicU64,
	reads_cache: AtomicU64,
	bytes_read_cache: AtomicU64,
}

impl UsageStats {
	/// New empty usage stats.
	pub fn new() -> Self {
		Self {
			started: std::time::Instant::now(),
			reads: 0.into(),
			bytes_read: 0.into(),
			writes: 0.into(),
			bytes_written: 0.into(),
			reads_cache: 0.into(),
			bytes_read_cache: 0.into(),
		}
	}

	/// Tally one read operation, of some length.
	pub fn tally_read(&self, data_bytes: u64, cache: bool) {
		self.reads.fetch_add(1, AtomicOrdering::Relaxed);
		self.bytes_read.fetch_add(data_bytes, AtomicOrdering::Relaxed);
		if cache {
			self.reads_cache.fetch_add(1, AtomicOrdering::Relaxed);
			self.bytes_read_cache.fetch_add(data_bytes, AtomicOrdering::Relaxed);
		}
	}

	/// Tally one key read.
	pub fn tally_key_read(&self, key: &[u8], val: Option<Vec<u8>>, cache: bool) -> Option<Vec<u8>> {
		self.tally_read(key.len() as u64 + val.as_ref().map(|x| x.len() as u64).unwrap_or(0), cache);
		val
	}

	/// Tally one child key read.
	pub fn tally_child_key_read(&self, key: &(Vec<u8>, Vec<u8>), val: Option<Vec<u8>>, cache: bool) -> Option<Vec<u8>> {
		self.tally_read(key.0.len() as u64 + key.1.len() as u64 + val.as_ref().map(|x| x.len() as u64).unwrap_or(0), cache);
		val
	}

	/// Tally some write operations, including their byte count.
	pub fn tally_writes(&self, ops: u64, data_bytes: u64) {
		self.writes.fetch_add(ops, AtomicOrdering::Relaxed);
		self.bytes_written.fetch_add(data_bytes, AtomicOrdering::Relaxed);
	}

	/// Merge state machine usage info.
	pub fn merge_sm(&self, info: sp_state_machine::UsageInfo) {
		self.reads.fetch_add(info.reads.ops, AtomicOrdering::Relaxed);
		self.bytes_read.fetch_add(info.reads.bytes, AtomicOrdering::Relaxed);
		self.writes.fetch_add(info.writes.ops, AtomicOrdering::Relaxed);
		self.bytes_written.fetch_add(info.writes.bytes, AtomicOrdering::Relaxed);
		self.reads_cache.fetch_add(info.cache_reads.ops, AtomicOrdering::Relaxed);
		self.bytes_read_cache.fetch_add(info.cache_reads.bytes, AtomicOrdering::Relaxed);
	}

	pub fn take(&self) -> sp_state_machine::UsageInfo {
		use sp_state_machine::UsageUnit;

		fn unit(ops: &AtomicU64, bytes: &AtomicU64) -> UsageUnit {
			UsageUnit { ops: ops.swap(0, AtomicOrdering::Relaxed), bytes: bytes.swap(0, AtomicOrdering::Relaxed) }
		}

		sp_state_machine::UsageInfo {
			reads: unit(&self.reads, &self.bytes_read),
			writes: unit(&self.writes, &self.bytes_written),
			cache_reads: unit(&self.reads_cache, &self.bytes_read_cache),
			memory: 0, // TODO:
			started: self.started,
			span: self.started.elapsed(),
		}
	}
}
