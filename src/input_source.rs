use anyhow::{Result, anyhow, bail};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    string::{CFString, CFStringRef},
};

pub const ABC_INPUT_SOURCE_ID: &str = "com.apple.keylayout.ABC";

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceID: CFStringRef;
    fn TISCreateInputSourceList(properties: CFDictionaryRef, include_all: u8) -> CFArrayRef;
    fn TISSelectInputSource(input_source: CFTypeRef) -> i32;
    fn TISCopyCurrentKeyboardInputSource() -> CFTypeRef;
    fn TISGetInputSourceProperty(input_source: CFTypeRef, property_key: CFStringRef) -> CFTypeRef;
}

/// Resolves the ABC keyboard input source. The returned [`CFType`] owns a retained
/// reference, so it stays valid until dropped.
pub fn resolve_abc() -> Result<CFType> {
    unsafe {
        let key = CFString::wrap_under_get_rule(kTISPropertyInputSourceID);
        let value = CFString::new(ABC_INPUT_SOURCE_ID);
        let filter = CFDictionary::from_CFType_pairs(&[(key, value)]);

        let list_ref = TISCreateInputSourceList(filter.as_concrete_TypeRef(), 0);
        if list_ref.is_null() {
            bail!("TISCreateInputSourceList returned null");
        }
        let array: CFArray<CFTypeRef> = CFArray::wrap_under_create_rule(list_ref);
        let first = array
            .get(0)
            .ok_or_else(|| anyhow!("{ABC_INPUT_SOURCE_ID} not installed"))?;

        // The CFArray retains its elements; wrap_under_get_rule retains again so the source
        // outlives the array.
        Ok(CFType::wrap_under_get_rule(*first))
    }
}

pub fn select(source: &CFType) -> Result<()> {
    match unsafe { TISSelectInputSource(source.as_CFTypeRef()) } {
        0 => Ok(()),
        status => bail!("TISSelectInputSource failed with status {status}"),
    }
}

/// Returns the identifier of the currently-active keyboard input source, e.g.
/// `"com.apple.keylayout.ABC"`. Returns `None` if the system call fails.
pub fn current_id() -> Option<String> {
    unsafe {
        let src_ref = TISCopyCurrentKeyboardInputSource();
        if src_ref.is_null() {
            return None;
        }
        let src = CFType::wrap_under_create_rule(src_ref);
        let id_ref =
            TISGetInputSourceProperty(src.as_CFTypeRef(), kTISPropertyInputSourceID) as CFStringRef;
        (!id_ref.is_null()).then(|| CFString::wrap_under_get_rule(id_ref).to_string())
    }
}
