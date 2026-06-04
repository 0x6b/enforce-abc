use std::{cell::RefCell, ptr::NonNull};

use anyhow::{Result, anyhow};
use block2::RcBlock;
use core_foundation::base::CFType;
use log::{debug, error, info};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace,
    NSWorkspaceApplicationKey,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObjectProtocol, NSOperationQueue, NSString,
};

use crate::input_source::{ABC_INPUT_SOURCE_ID, current_id, resolve_abc, select};

pub fn run() -> Result<()> {
    let mtm =
        MainThreadMarker::new().ok_or_else(|| anyhow!("must be called from the main thread"))?;

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let source = RefCell::new(resolve_abc()?);
    if let Err(why) = enforce_abc(&source, "<startup>") {
        error!("Failed to switch to {ABC_INPUT_SOURCE_ID} at startup: {why}");
    }

    let block = RcBlock::new(move |note: NonNull<NSNotification>| {
        let trigger = activated_app_label(unsafe { note.as_ref() });
        if let Err(why) = enforce_abc(&source, &trigger) {
            error!("Failed to switch to {ABC_INPUT_SOURCE_ID} (trigger: {trigger}): {why}");
        }
    });

    let _observer = observe_app_activation(&block);
    info!("Listening for app activation events. Forcing {ABC_INPUT_SOURCE_ID}.");

    app.run();
    Ok(())
}

fn enforce_abc(source: &RefCell<CFType>, trigger: &str) -> Result<()> {
    let before = current_id();
    if before.as_deref() == Some(ABC_INPUT_SOURCE_ID) {
        debug!("Trigger: {trigger}; already on {ABC_INPUT_SOURCE_ID}, no-op");
        return Ok(());
    }
    if select(&source.borrow()).is_err() {
        debug!("Trigger: {trigger}; cached source stale, re-resolving");
        *source.borrow_mut() = resolve_abc()?;
        select(&source.borrow())?;
    }
    debug!(
        "Trigger: {trigger}; switched {} -> {ABC_INPUT_SOURCE_ID}",
        before.as_deref().unwrap_or("<unknown>"),
    );
    Ok(())
}

/// Extract the activated application from the notification's `userInfo`, falling back to
/// `NSWorkspace.frontmostApplication`.
fn activated_app_label(note: &NSNotification) -> String {
    let app = note
        .userInfo()
        .and_then(|info| unsafe { info.objectForKey(NSWorkspaceApplicationKey) })
        .and_then(|obj| obj.downcast::<NSRunningApplication>().ok())
        .or_else(|| NSWorkspace::sharedWorkspace().frontmostApplication());

    let Some(app) = app else { return "<no app>".into() };
    let name = app
        .localizedName()
        .map_or_else(|| "<unnamed>".into(), |s| s.to_string());
    let bundle = app
        .bundleIdentifier()
        .map_or_else(|| "<no bundle id>".into(), |s| s.to_string());
    let pid = app.processIdentifier();
    format!("{name} ({bundle}, pid {pid})")
}

fn observe_app_activation(
    block: &RcBlock<dyn Fn(NonNull<NSNotification>)>,
) -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    let name = NSString::from_str("NSWorkspaceDidActivateApplicationNotification");
    let queue = NSOperationQueue::mainQueue();
    unsafe {
        center.addObserverForName_object_queue_usingBlock(Some(&name), None, Some(&queue), block)
    }
}
