//! WebKit, offscreen: a WKWebView in a window off the screen loads the
//! page, then prints it to a PDF file through AppKit's print operation
//! with its panels off, which pages it as the paper allows. createPDF,
//! which makes one page as tall as the note, is the fallback. All on
//! the main thread, as AppKit and WebKit require.

use std::cell::RefCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool, NSObject};
use objc2::{DefinedClass as _, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSPrintInfo, NSPrintJobSavingURL, NSPrintOperation, NSPrintSaveJob,
    NSPrintingPaginationMode, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSCopying as _, NSData, NSError, NSPoint, NSRect, NSSize, NSString, NSURL};
use objc2_web_kit::{WKWebView, WKWebViewConfiguration};

/// The view's size in points while it lays the page out: A4's width at
/// 96 points an inch, roughly. Printing lays it out again for the paper.
const VIEW: NSSize = NSSize::new(794., 1123.);
/// The paper's margins, in points: three quarters of an inch.
const MARGIN: f64 = 54.;

define_class!(
    /// Hears when a print operation has run.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "KansoPrintDelegate"]
    #[ivars = RefCell<Option<async_channel::Sender<bool>>>]
    struct PrintDelegate;

    impl PrintDelegate {
        #[unsafe(method(printOperationDidRun:success:contextInfo:))]
        fn did_run(&self, _operation: &NSPrintOperation, success: Bool, _context: *mut c_void) {
            if let Some(done) = self.ivars().borrow_mut().take() {
                done.try_send(success.as_bool()).ok();
            }
        }
    }
);

impl PrintDelegate {
    fn new(mtm: MainThreadMarker, done: async_channel::Sender<bool>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(Some(done)));
        // SAFETY: NSObject's init, on a freshly allocated object.
        unsafe { msg_send![super(this), init] }
    }
}

/// The page, laid out offscreen. Kept until its PDF is made: dropping
/// it closes the window and lets the view go.
pub struct Page {
    view: Retained<WKWebView>,
    window: Retained<NSWindow>,
    delegate: RefCell<Option<Retained<PrintDelegate>>>,
}

impl Drop for Page {
    fn drop(&mut self) {
        self.window.close();
    }
}

impl Page {
    /// A web view loading `html`, in a window off the screen (printing
    /// needs the view to be in one; it is never shown).
    pub fn load(html: &str) -> Result<Page, String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        let frame = NSRect::new(NSPoint::new(-20_000., -20_000.), VIEW);
        // SAFETY: on the main thread (checked above). The window isn't
        // released when closed, as `Page` owns it; the configuration
        // is copied by the view.
        unsafe {
            let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                mtm.alloc::<NSWindow>(),
                frame,
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            );
            window.setReleasedWhenClosed(false);
            let configuration = WKWebViewConfiguration::new(mtm);
            let view = WKWebView::initWithFrame_configuration(
                mtm.alloc::<WKWebView>(),
                NSRect::new(NSPoint::new(0., 0.), VIEW),
                &configuration,
            );
            window.setContentView(Some(&view));
            view.loadHTMLString_baseURL(&NSString::from_str(html), None);
            Ok(Page {
                view,
                window,
                delegate: RefCell::new(None),
            })
        }
    }

    pub fn is_loading(&self) -> bool {
        // SAFETY: a property read, on the main thread (GPUI's foreground).
        unsafe { self.view.isLoading() }
    }

    /// Starts printing the page to a PDF at `path`, on the user's
    /// default paper with even margins, as wide as the paper and as
    /// many pages long as it takes; `done` hears whether it worked.
    pub fn print(&self, path: &Path, done: async_channel::Sender<bool>) -> Result<(), String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        let path = path.to_str().ok_or("the file's path isn't valid text")?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        // A copy of the shared print info: the user's paper, with the
        // app's own settings left as they were.
        let info: Retained<NSPrintInfo> = NSPrintInfo::sharedPrintInfo().copy();
        info.setTopMargin(MARGIN);
        info.setBottomMargin(MARGIN);
        info.setLeftMargin(MARGIN);
        info.setRightMargin(MARGIN);
        info.setHorizontalPagination(NSPrintingPaginationMode::Fit);
        info.setVerticalPagination(NSPrintingPaginationMode::Automatic);
        info.setHorizontallyCentered(false);
        info.setVerticallyCentered(false);
        let delegate = PrintDelegate::new(mtm, done);
        // SAFETY: on the main thread. The dictionary holds print info
        // attributes, and the saving URL's value is an NSURL, as AppKit
        // expects. The delegate answers the selector given, with the
        // signature AppKit calls it with, and is kept (in `self`) until
        // the operation is over.
        unsafe {
            info.setJobDisposition(NSPrintSaveJob);
            let url: &AnyObject = &url;
            info.dictionary().insert(NSPrintJobSavingURL, url);
            let operation = self.view.printOperationWithPrintInfo(&info);
            operation.setShowsPrintPanel(false);
            operation.setShowsProgressPanel(false);
            // WebKit's printing view starts with no size, and prints
            // nothing until it has one.
            if let Some(view) = operation.view() {
                view.setFrame(self.view.bounds());
            }
            operation.runOperationModalForWindow_delegate_didRunSelector_contextInfo(
                &self.window,
                Some(&delegate),
                Some(sel!(printOperationDidRun:success:contextInfo:)),
                std::ptr::null_mut(),
            );
        }
        *self.delegate.borrow_mut() = Some(delegate);
        Ok(())
    }

    /// Asks WebKit for the PDF of the whole page as one page and writes
    /// it to `path`; `done` hears how it went.
    pub fn create_pdf(&self, path: PathBuf, done: async_channel::Sender<Result<(), String>>) {
        let handler = RcBlock::new(move |data: *mut NSData, error: *mut NSError| {
            // SAFETY: WebKit passes a valid NSData or NSError, or null.
            let result = unsafe {
                match (data.as_ref(), error.as_ref()) {
                    (Some(data), _) => {
                        std::fs::write(&path, data.to_vec()).map_err(|err| err.to_string())
                    }
                    (None, Some(error)) => Err(error.localizedDescription().to_string()),
                    (None, None) => Err("WebKit made no PDF".into()),
                }
            };
            done.try_send(result).ok();
        });
        // SAFETY: on the main thread; WebKit retains the handler until it
        // has called it.
        unsafe {
            self.view
                .createPDFWithConfiguration_completionHandler(None, &handler)
        };
    }
}
