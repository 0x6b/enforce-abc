use std::os::raw::c_void;

use anyhow::{Result, anyhow, bail};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    dictionary::CFDictionary,
    string::{CFString, CFStringRef},
};

pub const ABC_INPUT_SOURCE_ID: &str = "com.apple.keylayout.ABC";

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceID: CFStringRef;
    fn TISCreateInputSourceList(properties: *const c_void, includeAllInstalled: u8) -> CFArrayRef;
    fn TISSelectInputSource(input_source: *const c_void) -> i32;
    fn TISCopyCurrentKeyboardInputSource() -> *const c_void;
    fn TISGetInputSourceProperty(
        input_source: *const c_void,
        property_key: CFStringRef,
    ) -> *const c_void;
}

/// Resolves the ABC keyboard input source. The returned [`CFType`] owns a retained
/// reference, so it stays valid until dropped.
pub fn resolve_abc() -> Result<CFType> {
    unsafe {
        let key = CFString::wrap_under_get_rule(kTISPropertyInputSourceID);
        let value = CFString::new(ABC_INPUT_SOURCE_ID);
        let filter = CFDictionary::from_CFType_pairs(&[(key, value)]);

        let list_ref = TISCreateInputSourceList(filter.as_concrete_TypeRef() as *const c_void, 0);
        if list_ref.is_null() {
            bail!("TISCreateInputSourceList returned null");
        }
        let array: CFArray<*const c_void> = CFArray::wrap_under_create_rule(list_ref);
        let first = array
            .get(0)
            .ok_or_else(|| anyhow!("{ABC_INPUT_SOURCE_ID} not installed"))?;

        // The CFArray retains its elements; wrap_under_get_rule retains again so the source
        // outlives the array.
        Ok(CFType::wrap_under_get_rule(*first as CFTypeRef))
    }
}

pub fn select(source: &CFType) -> Result<()> {
    let status = unsafe { TISSelectInputSource(source.as_CFTypeRef()) };
    if status == 0 { Ok(()) } else { bail!("TISSelectInputSource failed with status {status}") }
}

/// Returns the identifier of the currently-active keyboard input source, e.g.
/// `"com.apple.keylayout.ABC"`. Returns `None` if the system call fails.
pub fn current_id() -> Option<String> {
    unsafe {
        let src_ref = TISCopyCurrentKeyboardInputSource();
        if src_ref.is_null() {
            return None;
        }
        let src = CFType::wrap_under_create_rule(src_ref as CFTypeRef);
        let id_ref =
            TISGetInputSourceProperty(src.as_CFTypeRef(), kTISPropertyInputSourceID) as CFStringRef;
        (!id_ref.is_null()).then(|| CFString::wrap_under_get_rule(id_ref).to_string())
    }
}
