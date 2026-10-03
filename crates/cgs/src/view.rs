use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::{MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSAccessibility, NSControl, NSControlSize, NSLayoutAttribute, NSLayoutConstraint,
    NSLayoutRelation, NSUserInterfaceItemIdentification, NSView, NSViewController,
};
use objc2_foundation::NSString;

use crate::{Insets, Ui};

pub trait NativeView: 'static {
    fn ns_view(&self) -> &NSView;
    fn view_controller(&self) -> Option<&objc2_app_kit::NSViewController> { None }
    fn set_hidden(&self, value: bool) { self.ns_view().setHidden(value); }
    fn tooltip(&self, text: &str) { self.ns_view().setToolTip(Some(&NSString::from_str(text))); }
    fn identifier(&self, text: &str) {
        self.ns_view().setIdentifier(Some(&NSString::from_str(text)));
    }
    fn accessibility_label(&self, text: &str) {
        self.ns_view().setAccessibilityLabel(Some(&NSString::from_str(text)));
    }
    fn accessibility_help(&self, text: &str) {
        self.ns_view().setAccessibilityHelp(Some(&NSString::from_str(text)));
    }
    fn accessibility_identifier(&self, text: &str) {
        self.ns_view().setAccessibilityIdentifier(Some(&NSString::from_str(text)));
    }
    fn width(&self, value: f64) {
        dimension(
            self.ns_view(),
            NSLayoutAttribute::Width,
            NSLayoutRelation::Equal,
            value,
        );
    }
    fn height(&self, value: f64) {
        dimension(
            self.ns_view(),
            NSLayoutAttribute::Height,
            NSLayoutRelation::Equal,
            value,
        );
    }
    fn min_width(&self, value: f64) {
        dimension(
            self.ns_view(),
            NSLayoutAttribute::Width,
            NSLayoutRelation::GreaterThanOrEqual,
            value,
        );
    }
    fn hugging_priority(&self, priority: f32, axis: objc2_app_kit::NSLayoutConstraintOrientation) {
        self.ns_view().setContentHuggingPriority_forOrientation(priority, axis);
    }
    fn compression_resistance(
        &self,
        priority: f32,
        axis: objc2_app_kit::NSLayoutConstraintOrientation,
    ) {
        self.ns_view()
            .setContentCompressionResistancePriority_forOrientation(priority, axis);
    }
    fn max_width(&self, value: f64) {
        dimension(
            self.ns_view(),
            NSLayoutAttribute::Width,
            NSLayoutRelation::LessThanOrEqual,
            value,
        );
    }
}
pub trait NativeControl: NativeView {
    fn ns_control(&self) -> &NSControl;
    fn set_enabled(&self, value: bool) { self.ns_control().setEnabled(value); }
    fn control_size(&self, size: NSControlSize) { self.ns_control().setControlSize(size); }
}
impl NativeView for Retained<NSView> {
    fn ns_view(&self) -> &NSView { self }
}

impl<T: NativeView + ?Sized> NativeView for std::rc::Rc<T> {
    fn ns_view(&self) -> &NSView { (**self).ns_view() }

    fn view_controller(&self) -> Option<&NSViewController> { (**self).view_controller() }
}
impl<T: NativeControl> NativeControl for std::rc::Rc<T> {
    fn ns_control(&self) -> &NSControl { (**self).ns_control() }
}

pub(crate) fn dimension(
    view: &NSView,
    attribute: NSLayoutAttribute,
    relation: NSLayoutRelation,
    value: f64,
) {
    view.setTranslatesAutoresizingMaskIntoConstraints(false);
    unsafe { NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(
        view, attribute, relation, None, NSLayoutAttribute::NotAnAttribute, 1.0, value
    ) }.setActive(true);
}
pub(crate) fn pin(parent: &NSView, child: &NSView, insets: Insets) {
    child.setTranslatesAutoresizingMaskIntoConstraints(false);
    for (attribute, constant) in [
        (NSLayoutAttribute::Leading, insets.left),
        (NSLayoutAttribute::Trailing, -insets.right),
        (NSLayoutAttribute::Top, insets.top),
        (NSLayoutAttribute::Bottom, -insets.bottom),
    ] {
        unsafe { NSLayoutConstraint::constraintWithItem_attribute_relatedBy_toItem_attribute_multiplier_constant(child, attribute, NSLayoutRelation::Equal, Some(parent), attribute, 1.0, constant) }.setActive(true);
    }
}
pub(crate) fn prepare(view: &NSView) { view.setTranslatesAutoresizingMaskIntoConstraints(false); }

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "CgUiDocumentView"]
    struct DocumentView;
    impl DocumentView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self)->bool{true}
    }
);

pub struct View {
    native: Retained<NSView>,
    children: RefCell<Vec<Box<dyn NativeView>>>,
}
impl View {
    pub fn new(ui: &Ui) -> Self {
        Self {
            native: NSView::new(ui.mtm()),
            children: RefCell::new(Vec::new()),
        }
    }

    pub fn flipped(ui: &Ui) -> Self {
        let native: Retained<DocumentView> =
            unsafe { msg_send![super(DocumentView::alloc(ui.mtm()).set_ivars(())), init] };
        Self {
            native: native.into_super(),
            children: RefCell::new(Vec::new()),
        }
    }

    pub fn ns_view(&self) -> &NSView { &self.native }

    pub fn content(self, child: impl NativeView, insets: Insets) -> Self {
        self.native.addSubview(child.ns_view());
        pin(&self.native, child.ns_view(), insets);
        self.children.borrow_mut().push(Box::new(child));
        self
    }
}
impl NativeView for View {
    fn ns_view(&self) -> &NSView { &self.native }
}

pub struct PageHost {
    view: View,
    controller: Retained<NSViewController>,
    page: RefCell<Option<Box<dyn NativeView>>>,
}
impl PageHost {
    pub fn new(ui: &Ui) -> Self {
        let view = View::new(ui);
        let controller = NSViewController::new(ui.mtm());
        controller.setView(view.ns_view());
        Self {
            view,
            controller,
            page: RefCell::new(None),
        }
    }

    pub fn ns_view_controller(&self) -> &NSViewController { &self.controller }

    pub fn set_page(&self, page: impl NativeView) {
        if let Some(old) = self.page.borrow_mut().take() {
            old.ns_view().removeFromSuperview();
            if let Some(controller) = old.view_controller() {
                controller.removeFromParentViewController();
            }
        }
        if let Some(controller) = page.view_controller() {
            self.controller.addChildViewController(controller);
        }
        self.view.ns_view().addSubview(page.ns_view());
        pin(self.view.ns_view(), page.ns_view(), Insets {
            top: 0.0,
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
        });
        *self.page.borrow_mut() = Some(Box::new(page));
    }
}
impl NativeView for PageHost {
    fn view_controller(&self) -> Option<&NSViewController> { Some(&self.controller) }

    fn ns_view(&self) -> &NSView { self.view.ns_view() }
}

pub struct Metrics;
impl Metrics {
    pub const CONTROL_SPACING: f64 = 8.0;
    pub const DETAIL_MIN_WIDTH: f64 = 400.0;
    pub const PAGE_INSET: f64 = 24.0;
    pub const PAGE_TOP_INSET: f64 = 20.0;
    pub const ROW_SPACING: f64 = 12.0;
    pub const SECTION_SPACING: f64 = 24.0;
    pub const SIDEBAR_MIN_WIDTH: f64 = 160.0;
    pub const SIDEBAR_WIDTH: f64 = 200.0;
}
