//	mem/heap.rs (ministd crate)
//	this file originally belonged to baseOS project
//		an OS template on which to build


//  this file implements features of the buddy_system_allocator

/// tells the allocator how to align data  
/// 
/// this is also the default align for all allocations  
/// if you change the value:
/// - must be > 0
/// - must be power of 2
/// 
/// otherwise it could break things
//pub const ALLOC_ALIGN: usize = 4;

pub use buddy_system_allocator as allocator;
use spin::MutexGuard;
use core::alloc::GlobalAlloc;
pub use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::{copy_nonoverlapping, drop_in_place, null_mut, NonNull};
use crate::mem::Region;
use crate::spin::Mutex;
use crate::Immutable;

pub type LockedHeap = allocator::LockedHeap<32>;
pub type Heap = allocator::Heap<32>;
pub struct Allocator {
    alloc: LockedHeap,
    regions: Mutex<Region>,     //  TODO: use vector
}

impl Allocator {
    pub(crate) const fn new() -> Self {
        Self {
            alloc: allocator::LockedHeap::new(),
            regions: Mutex::new(Region::empty()),
        }
    }

    /// allocates data of type T with proper alignment
    /// - layout: `size: size_of::<T>(), align: align_of::<T>()`
    #[inline]
    pub unsafe fn allocate<T: Sized>(&self, val: T) -> Result<NonNull<T>, ()> {
        let layout = Layout::new::<T>();

        if let Ok(d) = self.alloc.lock().alloc(layout) {
            let data = unsafe { NonNull::new_unchecked(d.as_ptr() as *mut T) };
            unsafe { *data.as_ptr() = val };
            
            Ok(data)
        } else {
            Err(())
        }
    }

    /// allocates uninitialized data of type T with proper alignment
    /// - layout: `size: size_of::<T>(), align: align_of::<T>()`
    #[inline]
    pub unsafe fn allocate_uninit<T: Sized>(&self) -> Result<NonNull<MaybeUninit<T>>, ()> {
        let layout = unsafe { Layout::from_size_align_unchecked(size_of::<T>(), align_of::<T>()) };
        if let Ok(d) = self.alloc.lock().alloc(layout) {
            let data = unsafe { NonNull::new_unchecked(d.as_ptr() as *mut MaybeUninit<T>) };
            Ok(data)
        } else {
            Err(())
        }
    }

    /// deallocate pointer from heap and run its `drop` if it is needed
    /// - layout: `size: size_of::<T>(), align: align_od::<T>()`
    #[inline]
    pub unsafe fn delete<T: Sized>(&self, ptr: NonNull<T>) {
        unsafe {
            drop_in_place(ptr.as_ptr());
            self.dealloc(ptr.as_ptr() as *mut u8, Layout::from_size_align_unchecked(size_of::<T>(), align_of::<T>()));
        }
    }

}

impl Allocator {


    /// gets immutable reference to regions
    #[inline]
    pub fn get_regions(&self) -> Immutable<MutexGuard<Region>> {
        Immutable::new(self.regions.lock())
    }
    
    /// try to obtain regions
    #[inline]
    pub fn try_get_regions(&self) -> Option<Immutable<MutexGuard<Region>>> {
        if let Some(guard) = self.regions.try_lock() {
            Some(Immutable::new(guard))
        } else {
            None
        }
    }

    /// add range of addresses to heap  
    /// also pushes into the regions vector
    #[inline(always)]
    pub unsafe fn add_to_heap(&self, start: usize, end: usize) {
        //  once using vector for regions: push
        unsafe { self.alloc.lock().add_to_heap(start, end) };
    }

    #[inline(always)]
    pub unsafe fn add_to_heap_locked(&self, guard: &mut MutexGuard<Heap>, start: usize, end: usize) {
        //  push into vector
        unsafe { guard.add_to_heap(start, end) };
    }

    /// returns the actual number of bytes in the heap
    #[inline(always)]
    pub fn total_bytes(&self) -> usize {
        self.alloc.lock().stats_total_bytes()
    }

    /// returns the number of bytes that are allocated
    #[inline(always)]
    pub fn allocated_bytes(&self) -> usize {
        self.alloc.lock().stats_alloc_actual()
    }

    /// reallocates memory to an new layout
    pub unsafe fn realloc_layout(&self, old: *mut u8, old_l: Layout, new_l: Layout) -> *mut u8 {
        let new = unsafe { self.alloc.alloc(new_l) };

        if new.is_null() {
            return core::ptr::null_mut();
        }

        unsafe {
            core::ptr::copy_nonoverlapping(old, new, core::cmp::min(old_l.size(), new_l.size()));
            ALLOCATOR.dealloc(old, old_l);
        }

        new

    }
    
}



unsafe impl GlobalAlloc for Allocator {

    /// allocates new data on the heap  
    /// if allocation fails:
    /// - runs the `out_of_memory_handler` routine (defined in main crate)
    ///   - success: try allocation again
    ///   - failure: returns null
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        crate::println!("ALLOC");

        match self.alloc.lock().alloc(layout) {
            Ok(data) => data.as_ptr(),
            Err(_) => {
                //  run out_of_memory routine and try again
                
                let mut alloc = self.alloc.lock();
                if let Ok(_) = unsafe { out_of_memory_handler(&mut alloc, &self) }{
                    match alloc.alloc(layout) {
                        Ok(data) => data.as_ptr(),
                        Err(_) => null_mut(),
                    }
                } else {
                    null_mut()
                }

            },
        }
    }

    /// same as `alloc` but zeroes the allocated buffer
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match self.alloc.lock().alloc(layout) {
            Ok(d) => {
                unsafe { core::ptr::write_bytes(d.as_ptr() as *mut usize, 0, layout.size()/size_of::<usize>()) };
                d.as_ptr()
            },
            Err(_) => {
                //  run out_of_memory routine and try again
                
                let mut alloc = self.alloc.lock();
                if let Ok(_) = unsafe { out_of_memory_handler(&mut alloc, &self) } {
                    match alloc.alloc(layout) {
                        Ok(d) => {
                            unsafe { core::ptr::write_bytes(d.as_ptr() as *mut usize, 0, layout.size()/size_of::<usize>()) };
                            d.as_ptr()
                        }
                        Err(_) => null_mut(),
                    }
                } else {
                    null_mut()
                }
            },
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.alloc.lock().dealloc(unsafe { NonNull::new_unchecked(ptr) }, layout);
    }

    /// reallocates memory
    /// - does not deallocate the old buffer if allocation fails
    /// 
    /// used layout: `Layout::from_size_unchecked(new_size, layout.align())`
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = match self.alloc.lock().alloc(unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) } ) {
            Ok(data) => data.as_ptr(),
            Err(_) => return null_mut(),
        };

        let count = core::cmp::min(new_size, layout.size());
        unsafe {
            copy_nonoverlapping(ptr, new, count);
            self.dealloc(ptr, layout);
        }
        new
    }

}


#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();
pub static REGIONS: Mutex<Region> = Mutex::new(Region::empty());
    // use Vec later


unsafe extern "Rust" {

    //  functions defined by the developer in the main crate

     pub(crate) fn find_heap_region() -> Result<Region, ()>;
     pub(crate) fn out_of_memory_handler(heap: &mut MutexGuard<Heap>, allocator: &Allocator) -> Result<(), ()>;
}



//  TODO: use mutex<Vec<Region>> for heap mapping



/// `init` initializes heap  
/// **IMPORTANT**
/// - this function uses the [`mem::find_heap_region`] function from the main crate
///   - rewrite this function to change the default behaviour
/// 
/// You can check where the heap is with the [`ministd::mem::heap::REGION`] variable
/// - please do not change it
pub(crate) fn init() -> Result<(), ()> {

    if let Ok(reg) = unsafe { find_heap_region() } {
        *REGIONS.lock() = reg;
        let mut alloc = ALLOCATOR.alloc.lock();
        unsafe { alloc.init(reg.start, reg.size); }
        return Ok(());
    }
    Err(())

}