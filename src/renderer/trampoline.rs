use crate::renderer::bridge::ffi;
use cxx::ExternType;
use cxx::type_id;

// Generic Callback objects
// The function passed to this object is called by the corresponding object

// Trampoline function called from C++ with data the function pointer to the closure
extern "C" fn voidTrampolineFunction(data: *mut i8) {
    let closure = unsafe { &mut *(data as *mut Box<dyn Fn()>) };
    closure();
}

#[derive(Debug)]
pub struct VoidTrampoline {
    f: extern "C" fn(*mut i8),
    data: *mut i8,
}

impl VoidTrampoline {
    pub fn new<F: Fn() + 'static>(f: F) -> Self {
        // Due to dyn Fn(), the Box `boxed` is a fatpointer (ptr + vtable)
        // We have to convert first to a light pointer because otherwise
        // we are not able to store all data in a raw pointer
        let boxed: Box<dyn Fn()> = Box::new(f);
        let data = Box::into_raw(Box::new(boxed)) as *mut i8;
        Self { f: voidTrampolineFunction, data }
    }
}

unsafe impl ExternType for VoidTrampoline {
    type Id = type_id!(mln::bridge::VoidTrampoline); // Name must match with the one on the C++ side
    type Kind = cxx::kind::Trivial;
}

// #############################################################################

// bridge_callback!(
//     DidFinishRenderingFrameCallback,
//     extern "C" fn(needs_repaint: bool, placement_changed: bool)
// );

extern "C" fn failingLoadingMapTrampolineFunction(
    data: *mut i8,
    error: ffi::MapLoadError,
    message: &str,
) {
    let closure = unsafe { &mut *(data as *mut Box<dyn Fn(ffi::MapLoadError, &str)>) };
    closure(error, message);
}

#[derive(Debug)]
pub struct FailingLoadingMapTrampoline {
    f: extern "C" fn(*mut i8, ffi::MapLoadError, &str),
    data: *mut i8,
}

impl FailingLoadingMapTrampoline {
    pub fn new<F: Fn(ffi::MapLoadError, &str) + 'static>(f: F) -> Self {
        let boxed: Box<F> = Box::new(f);
        let data = Box::into_raw(Box::new(boxed)) as *mut i8;
        Self { f: failingLoadingMapTrampolineFunction, data }
    }
}

unsafe impl ExternType for FailingLoadingMapTrampoline {
    type Id = type_id!(mln::bridge::FailingLoadingMapTrampoline); // Name must match with the one on the C++ side
    type Kind = cxx::kind::Trivial;
}

// #############################################################################

extern "C" fn didFinishRenderingFrameTrampolineFunction(
    data: *mut i8,
    needs_repaint: bool,
    placement_changed: bool,
) {
    let closure = unsafe { &mut *(data as *mut Box<dyn Fn(bool, bool)>) };
    closure(needs_repaint, placement_changed);
}

#[derive(Debug)]
pub struct DidFinishRenderingFrameTrampoline {
    f: extern "C" fn(data: *mut i8, needs_repaint: bool, placement_changed: bool),
    data: *mut i8,
}

impl DidFinishRenderingFrameTrampoline {
    pub fn new<F: Fn(bool, bool) + 'static>(f: F) -> Self {
        let boxed: Box<F> = Box::new(f);
        let data = Box::into_raw(Box::new(boxed)) as *mut i8;
        Self { f: didFinishRenderingFrameTrampolineFunction, data }
    }
}

unsafe impl ExternType for DidFinishRenderingFrameTrampoline {
    type Id = type_id!(mln::bridge::DidFinishRenderingFrameTrampoline); // Name must match with the one on the C++ side
    type Kind = cxx::kind::Trivial;
}
