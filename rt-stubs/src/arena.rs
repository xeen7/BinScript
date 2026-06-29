use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

const DEFAULT_ARENA_CAPACITY: usize = 4096;

#[repr(C)]
pub struct DtorEntry {
    pub obj_ptr: *mut u8,
    pub drop_fn: unsafe extern "C-unwind" fn(*mut u8),
    pub next: *mut DtorEntry,
}

#[repr(C)]
pub struct Arena {
    base: *mut u8,
    bump: *mut u8,
    end: *mut u8,
    segments: *mut Segment,
    dtor_list: *mut DtorEntry,
}

#[repr(C)]
pub struct Segment {
    base: *mut u8,
    size: usize,
    next: *mut Segment,
}

#[no_mangle]
pub unsafe extern "C-unwind" fn arena_create(initial_capacity: usize) -> *mut Arena {
    let capacity = if initial_capacity == 0 { DEFAULT_ARENA_CAPACITY } else { initial_capacity };
    
    // Allocate the Arena struct itself
    let arena_layout = Layout::new::<Arena>();
    let arena_ptr = alloc(arena_layout) as *mut Arena;
    
    if arena_ptr.is_null() {
        std::alloc::handle_alloc_error(arena_layout);
    }
    
    // Allocate the initial segment
    let segment_layout = Layout::from_size_align(capacity, 8).unwrap();
    let base_ptr = alloc(segment_layout);
    
    if base_ptr.is_null() {
        std::alloc::handle_alloc_error(segment_layout);
    }
    
    (*arena_ptr).base = base_ptr;
    (*arena_ptr).bump = base_ptr;
    (*arena_ptr).end = base_ptr.add(capacity);
    (*arena_ptr).segments = ptr::null_mut();
    (*arena_ptr).dtor_list = ptr::null_mut();
    
    arena_ptr
}

#[no_mangle]
pub unsafe extern "C-unwind" fn arena_alloc(arena: *mut Arena, size: usize, align: usize) -> *mut u8 {
    let arena_ref = &mut *arena;
    
    // Align bump pointer
    let offset = arena_ref.bump.align_offset(align);
    let mut aligned_bump = arena_ref.bump.add(offset);
    
    // Check if we have enough space in the current segment
    if (aligned_bump as usize) + size <= (arena_ref.end as usize) {
        arena_ref.bump = aligned_bump.add(size);
        return aligned_bump;
    }
    
    // Need a new segment
    let current_capacity = (arena_ref.end as usize) - (arena_ref.base as usize);
    // Double the capacity for the next segment, but ensure it's at least `size`
    let mut next_capacity = current_capacity * 2;
    if next_capacity < size {
        next_capacity = size.next_power_of_two(); // simplified size rounding
    }
    
    // Save current segment
    let segment_layout = Layout::new::<Segment>();
    let segment_ptr = alloc(segment_layout) as *mut Segment;
    if segment_ptr.is_null() {
        std::alloc::handle_alloc_error(segment_layout);
    }
    
    (*segment_ptr).base = arena_ref.base;
    (*segment_ptr).size = current_capacity;
    (*segment_ptr).next = arena_ref.segments;
    arena_ref.segments = segment_ptr;
    
    // Allocate new segment
    let new_segment_layout = Layout::from_size_align(next_capacity, 8).unwrap();
    let new_base = alloc(new_segment_layout);
    if new_base.is_null() {
        std::alloc::handle_alloc_error(new_segment_layout);
    }
    
    arena_ref.base = new_base;
    arena_ref.end = new_base.add(next_capacity);
    
    // Align the new bump pointer (new_base is already aligned to 8, but might need more)
    let new_offset = new_base.align_offset(align);
    aligned_bump = new_base.add(new_offset);
    
    arena_ref.bump = aligned_bump.add(size);
    aligned_bump
}

#[no_mangle]
pub unsafe extern "C-unwind" fn arena_register_dtor(
    arena: *mut Arena,
    obj_ptr: *mut u8,
    drop_fn: unsafe extern "C-unwind" fn(*mut u8),
) {
    if arena.is_null() { return; }
    
    // Allocate DtorEntry inside the arena itself
    let dtor_ptr = arena_alloc(arena, std::mem::size_of::<DtorEntry>(), 8) as *mut DtorEntry;
    (*dtor_ptr).obj_ptr = obj_ptr;
    (*dtor_ptr).drop_fn = drop_fn;
    
    let arena_ref = &mut *arena;
    (*dtor_ptr).next = arena_ref.dtor_list;
    arena_ref.dtor_list = dtor_ptr;
}

#[no_mangle]
pub unsafe extern "C-unwind" fn arena_reset(arena: *mut Arena) {
    let arena_ref = &mut *arena;
    // Reset to the beginning of the CURRENT segment.
    // To properly reset, we'd ideally go back to the FIRST segment, but for Phase 4,
    // resetting just rewinds the bump pointer of the current segment.
    arena_ref.bump = arena_ref.base;
}

#[no_mangle]
pub unsafe extern "C-unwind" fn arena_destroy(arena: *mut Arena) {
    if arena.is_null() {
        return;
    }
    
    let arena_ref = &mut *arena;
    
    // Walk dtor_list in reverse allocation order and call drop_fns
    let mut current_dtor = arena_ref.dtor_list;
    while !current_dtor.is_null() {
        let entry = &*current_dtor;
        (entry.drop_fn)(entry.obj_ptr);
        current_dtor = entry.next;
    }
    
    // Free current segment
    let current_capacity = (arena_ref.end as usize) - (arena_ref.base as usize);
    let current_layout = Layout::from_size_align(current_capacity, 8).unwrap();
    dealloc(arena_ref.base, current_layout);
    
    // Free all previous segments
    let mut current_segment = arena_ref.segments;
    while !current_segment.is_null() {
        let seg_ref = &*current_segment;
        let next_segment = seg_ref.next;
        
        let seg_layout = Layout::from_size_align(seg_ref.size, 8).unwrap();
        dealloc(seg_ref.base, seg_layout);
        
        let struct_layout = Layout::new::<Segment>();
        dealloc(current_segment as *mut u8, struct_layout);
        
        current_segment = next_segment;
    }
    
    // Free Arena struct
    let arena_layout = Layout::new::<Arena>();
    dealloc(arena as *mut u8, arena_layout);
}
