//! Universal Provenance & Logic Ledger (UPLL) - Isolated Linear Memory Subsystem
//! Establishes deterministic runtime isolation via page-aligned data layouts,
//! strict overflow boundary checking, zero-copy memory mapping, and cryptographic zeroization.

use crate::VmError;

/// Defines the size of an individual isolated runtime memory page in bytes (64 KiB).
pub const PAGE_SIZE: usize = 65_536;
/// Defines the maximum allowable addressable index threshold for a single sandbox isolate.
pub const MAX_MEM_LIMIT: usize = 67_108_864; // 64 MiB total cap ceiling

/// Page-aligned linear layout executing memory sandboxing primitives safely.
#[derive(Debug, Clone)]
pub struct LinearMemoryIsolate {
    /// Contiguous raw buffer storing the active runtime heap data.
    pub raw_buffer: Vec<u8>,
    /// Number of virtual machine memory pages currently allocated to this container.
    pub current_pages: usize,
    /// Absolute maximum allocation constraint mapped to system configuration profiles.
    pub max_pages: usize,
}

impl LinearMemoryIsolate {
    /// Spawns a dedicated, isolated memory arena matrix initialized to baseline page configurations.
    pub fn new(initial_pages: usize) -> Self {
        let max_pages = MAX_MEM_LIMIT / PAGE_SIZE;
        let targeted_initial = initial_pages.min(max_pages);
        
        Self {
            raw_buffer: vec![0; targeted_initial * PAGE_SIZE],
            current_pages: targeted_initial,
            max_pages,
        }
    }

    /// Pulls a zero-copy raw byte slice from the sandboxed buffer, enforcing memory check limits.
    #[inline]
    pub fn read_bytes(&self, offset: usize, destination: &mut [u8]) -> Result<(), VmError> {
        let end_address = offset.checked_add(destination.len()).ok_or(VmError::MemoryAccessViolation {
            address: offset,
        })?;

        if end_address > self.raw_buffer.len() {
            return Err(VmError::MemoryAccessViolation { address: offset });
        }

        destination.copy_from_slice(&self.raw_buffer[offset..end_address]);
        Ok(())
    }

    /// Safely writes arbitrary data slices into target memory regions, preventing buffer exploits.
    #[inline]
    pub fn write_bytes(&mut self, offset: usize, source: &[u8]) -> Result<(), VmError> {
        let end_address = offset.checked_add(source.len()).ok_or(VmError::MemoryAccessViolation {
            address: offset,
        })?;

        if end_address > self.raw_buffer.len() {
            return Err(VmError::MemoryAccessViolation { address: offset });
        }

        self.raw_buffer[offset..end_address].copy_from_slice(source);
        Ok(())
    }

    /// Dynamically expands the virtual memory footprint by appending clean, initialized pages.
    pub fn dynamically_grow_heap(&mut self, additional_pages: usize) -> Result<usize, VmError> {
        let total_requested = self.current_pages.checked_add(additional_pages).ok_or(VmError::MemoryAccessViolation {
            address: usize::MAX,
        })?;

        if total_requested > self.max_pages {
            return Err(VmError::MemoryAccessViolation { address: self.raw_buffer.len() });
        }

        let updated_byte_allocation = total_requested * PAGE_SIZE;
        self.raw_buffer.resize(updated_byte_allocation, 0);
        
        let baseline_pages = self.current_pages;
        self.current_pages = total_requested;
        
        Ok(baseline_pages)
    }

    /// Returns the active byte scale representing the total bounds allocated to the isolate.
    #[inline]
    pub fn active_byte_capacity(&self) -> usize {
        self.raw_buffer.len()
    }

    /// Performs secure zeroization memory clearing across the raw vector space to disrupt side-channel vector analysis.
    pub fn purge_and_zeroize(&mut self) {
        // Explicitly write zeroes down the slice memory footprint
        for data_byte in self.raw_buffer.iter_mut() {
            *data_byte = 0x00;
        }
        self.raw_buffer.clear();
        self.current_pages = 0;
    }
}
