use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug)]
pub struct MonitoredRwLock<T> {
    inner: RwLock<T>,
    stats: Mutex<Stats>,
}

#[derive(Debug)]
pub struct ReadGuard<'a, T> {
    start: Instant,
    name: &'static str,
    inner: RwLockReadGuard<'a, T>,
    stats: &'a Mutex<Stats>,
}

#[derive(Debug)]
pub struct WriteGuard<'a, T> {
    start: Instant,
    name: &'static str,
    inner: RwLockWriteGuard<'a, T>,
    stats: &'a Mutex<Stats>,
}

#[derive(Debug, Default)]
struct Stats {
    named: HashMap<&'static str, LockStats>,
    read_wait: RunningAverage,
    write_wait: RunningAverage,
}

#[derive(Debug, Default)]
struct LockStats {
    read_usage: RunningAverage,
    write_usage: RunningAverage,
}

#[derive(Debug, Default)]
pub struct SummaryStats {
    pub read_wait: Option<Duration>,
    pub write_wait: Option<Duration>,
    pub read_usage: HashMap<&'static str, Duration>,
    pub write_usage: HashMap<&'static str, Duration>,
}

#[derive(Debug, Default)]
struct RunningAverage {
    sum: Duration,
    count: u32,
}

impl<T> MonitoredRwLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: RwLock::new(value),
            stats: Default::default(),
        }
    }

    pub fn read(&self, name: &'static str) -> ReadGuard<T> {
        let start_wait = Instant::now();
        let guard = self.inner.read().unwrap();
        let wait = start_wait.elapsed();
        self.stats.lock().unwrap().read_wait.push(wait);

        ReadGuard {
            start: Instant::now(),
            name,
            inner: guard,
            stats: &self.stats,
        }
    }

    pub fn write(&self, name: &'static str) -> WriteGuard<T> {
        let start_wait = Instant::now();
        let guard = self.inner.write().unwrap();
        let wait = start_wait.elapsed();
        self.stats.lock().unwrap().write_wait.push(wait);

        WriteGuard {
            start: Instant::now(),
            name,
            inner: guard,
            stats: &self.stats,
        }
    }

    pub fn pop_stats(&self) -> SummaryStats {
        let mut stats = self.stats.lock().unwrap();
        let mut read_usage = HashMap::new();
        let mut write_usage = HashMap::new();
        for (&name, x) in &mut stats.named {
            if let Some(avg) = x.read_usage.pop() {
                read_usage.insert(name, avg);
            }
            if let Some(avg) = x.write_usage.pop() {
                write_usage.insert(name, avg);
            }
        }

        SummaryStats {
            read_wait: stats.read_wait.pop(),
            write_wait: stats.write_wait.pop(),
            read_usage,
            write_usage,
        }
    }
}

impl RunningAverage {
    fn push(&mut self, sample: Duration) {
        self.sum += sample;
        self.count += 1;
    }

    fn pop(&mut self) -> Option<Duration> {
        let avg = (self.count > 0).then(|| self.sum / self.count);
        self.count = 0;
        self.sum = Duration::ZERO;
        avg
    }
}

impl<'a, T> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        let usage = self.start.elapsed();
        self.stats
            .lock()
            .unwrap()
            .named
            .entry(self.name)
            .or_default()
            .read_usage
            .push(usage);
    }
}

impl<'a, T> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        let usage = self.start.elapsed();
        self.stats
            .lock()
            .unwrap()
            .named
            .entry(self.name)
            .or_default()
            .write_usage
            .push(usage);
    }
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
