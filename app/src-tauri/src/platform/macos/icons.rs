#![expect(
    unsafe_code,
    reason = "AppKit bitmap allocation and PNG encoding use unsafe-marked initializers"
)]

//! Application icons rendered from the local bundle into PNG bytes.

use objc2::rc::{autoreleasepool, Retained};
use objc2::AllocAnyThread;
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSCalibratedRGBColorSpace, NSCompositingOperation,
    NSGraphicsContext, NSImage, NSImageInterpolation, NSWorkspace,
};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};

use crate::platform::{IconProvider, PlatformError};

const MIN_SIZE_PX: u32 = 16;
const MAX_SIZE_PX: u32 = 1024;
const OPERATION: &str = "render icon";
const BITS_PER_SAMPLE: isize = 8;
const SAMPLES_PER_PIXEL: isize = 4;

#[derive(Debug)]
pub(super) struct MacIcons;

impl IconProvider for MacIcons {
    fn app_icon_png(
        &self,
        bundle_id: &str,
        size_px: u32,
    ) -> Result<Option<Vec<u8>>, PlatformError> {
        let side = size_px.clamp(MIN_SIZE_PX, MAX_SIZE_PX);
        autoreleasepool(|_| render(bundle_id, side))
    }
}

fn render(bundle_id: &str, side: u32) -> Result<Option<Vec<u8>>, PlatformError> {
    let workspace = NSWorkspace::sharedWorkspace();
    let Some(url) = workspace.URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))
    else {
        return Ok(None);
    };
    let Some(path) = url.path() else {
        return Ok(None);
    };
    let icon = workspace.iconForFile(&path);
    draw_png(&icon, side).map(Some)
}

fn draw_png(icon: &NSImage, side: u32) -> Result<Vec<u8>, PlatformError> {
    let pixels = isize::try_from(side).map_err(|_| super::failed(OPERATION, "size overflow"))?;
    let rep = bitmap(pixels)?;
    draw_into(icon, &rep, side)?;
    encode_png(&rep)
}

fn draw_into(icon: &NSImage, rep: &NSBitmapImageRep, side: u32) -> Result<(), PlatformError> {
    let context = NSGraphicsContext::graphicsContextWithBitmapImageRep(rep)
        .ok_or_else(|| super::failed(OPERATION, "no graphics context"))?;
    let extent = f64::from(side);
    NSGraphicsContext::saveGraphicsState_class();
    NSGraphicsContext::setCurrentContext(Some(&context));
    context.setImageInterpolation(NSImageInterpolation::High);
    icon.drawInRect_fromRect_operation_fraction(
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(extent, extent)),
        NSRect::ZERO,
        NSCompositingOperation::SourceOver,
        1.0,
    );
    NSGraphicsContext::restoreGraphicsState_class();
    Ok(())
}

fn bitmap(pixels: isize) -> Result<Retained<NSBitmapImageRep>, PlatformError> {
    // SAFETY: null planes make AppKit allocate and own the pixel storage, and every other
    // argument describes an 8-bit RGBA, non-planar bitmap of `pixels` squared.
    let rep = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            pixels,
            pixels,
            BITS_PER_SAMPLE,
            SAMPLES_PER_PIXEL,
            true,
            false,
            NSCalibratedRGBColorSpace,
            0,
            0,
        )
    };
    rep.ok_or_else(|| super::failed(OPERATION, "bitmap allocation failed"))
}

fn encode_png(rep: &NSBitmapImageRep) -> Result<Vec<u8>, PlatformError> {
    // SAFETY: an empty dictionary is a valid properties argument for PNG encoding.
    let data = unsafe {
        rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    };
    data.map(|bytes| bytes.to_vec())
        .ok_or_else(|| super::failed(OPERATION, "png encoding failed"))
}
