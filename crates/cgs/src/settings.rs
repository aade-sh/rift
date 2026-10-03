use std::rc::Rc;

use objc2::rc::Retained;
use objc2_app_kit::{NSBox, NSBoxType, NSTextField, NSTitlePosition, NSView};

use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Validation {
    #[default]
    None,
    Info(String),
    Warning(String),
    Error(String),
}
impl Validation {
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Info(s) | Self::Warning(s) | Self::Error(s) => Some(s),
        }
    }
}
pub struct ValidationMessage {
    stack: HStack,
    label: SecondaryLabel,
}
impl ValidationMessage {
    pub fn new(ui: &Ui) -> Self {
        let label = SecondaryLabel::new(ui, "");
        let stack = HStack::new(ui);
        stack.ns_stack_view().addArrangedSubview(label.ns_view());
        let this = Self { stack, label };
        this.set_hidden(true);
        this
    }

    pub fn set_validation(&self, value: &Validation) {
        self.label.set_text(value.message().unwrap_or(""));
        self.set_hidden(matches!(value, Validation::None));
        self.accessibility_label(value.message().unwrap_or(""));
    }

    pub fn ns_text_field(&self) -> &NSTextField { self.label.ns_text_field() }
}
impl NativeView for ValidationMessage {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}
pub struct Badge(Caption);
impl Badge {
    pub fn new(ui: &Ui, text: &str) -> Self { Self(Caption::new(ui, text)) }

    pub fn set_text(&self, text: &str) { self.0.set_text(text); }

    pub fn ns_text_field(&self) -> &NSTextField { self.0.ns_text_field() }
}
impl NativeView for Badge {
    fn ns_view(&self) -> &NSView { self.0.ns_view() }
}

pub struct SettingsRow {
    ui: Ui,
    stack: VStack,
    line: Rc<HStack>,
    control: Box<dyn NativeView>,
    validation: Rc<ValidationMessage>,
}
impl SettingsRow {
    pub fn new(ui: &Ui, title: &str, control: impl NativeView) -> Self {
        Self::with_validation(ui, title, control, Rc::new(ValidationMessage::new(ui)))
    }

    pub fn with_validation(
        ui: &Ui,
        title: &str,
        control: impl NativeView,
        validation: Rc<ValidationMessage>,
    ) -> Self {
        let label = WrappingLabel::new(ui, title);
        let line = Rc::new(HStack::new(ui).push(label).spacer(ui));
        control.accessibility_label(title);
        line.ns_stack_view().addArrangedSubview(control.ns_view());
        let stack = VStack::new(ui).spacing(4.0).push(line.clone());
        stack.ns_stack_view().addArrangedSubview(validation.ns_view());
        Self {
            ui: *ui,
            stack,
            line,
            control: Box::new(control),
            validation,
        }
    }

    pub fn description(self, text: &str) -> Self {
        self.stack.add(SecondaryLabel::new(&self.ui, text));
        self
    }

    pub fn suffix(self, text: &str) -> Self {
        self.line.add(SecondaryLabel::new(&self.ui, text));
        self
    }

    pub fn set_validation(&self, value: &Validation) { self.validation.set_validation(value); }

    pub fn control_view(&self) -> &NSView { self.control.ns_view() }
}
impl NativeView for SettingsRow {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}
macro_rules! setting_row {
    ($name:ident) => {
        pub struct $name(SettingsRow);
        impl $name {
            pub fn new(ui: &Ui, title: &str, control: impl NativeControl) -> Self {
                Self(SettingsRow::new(ui, title, control))
            }

            pub fn description(self, text: &str) -> Self { Self(self.0.description(text)) }

            pub fn suffix(self, text: &str) -> Self { Self(self.0.suffix(text)) }

            pub fn set_validation(&self, value: &Validation) { self.0.set_validation(value); }

            pub fn control_view(&self) -> &NSView { self.0.control_view() }
        }
        impl NativeView for $name {
            fn ns_view(&self) -> &NSView { self.0.ns_view() }
        }
    };
}
setting_row!(SwitchRow);
setting_row!(PopupRow);
setting_row!(TextRow);
setting_row!(NumberRow);
setting_row!(SliderRow);
setting_row!(ColorRow);
setting_row!(ButtonRow);
pub type DisclosureRow = Disclosure;

/// A native grouping box with compact rows and system separators.
pub struct SettingsGroup {
    ui: Ui,
    native: Retained<NSBox>,
    rows: VStack,
}
impl SettingsGroup {
    pub fn new(ui: &Ui) -> Self {
        let native = NSBox::new(ui.mtm());
        native.setBoxType(NSBoxType::Custom);
        native.setTransparent(true);
        native.setBorderWidth(0.0);
        native.setTitlePosition(NSTitlePosition::NoTitle);
        native.setContentViewMargins(CGSize::new(0.0, 0.0));
        let rows = VStack::new(ui).spacing(0.0);
        native.setContentView(Some(rows.ns_view()));
        rows.ns_view()
            .widthAnchor()
            .constraintEqualToAnchor(&native.widthAnchor())
            .setActive(true);
        // NSBox owns its content view's frame; constrain its fitting size to the rows.
        rows.ns_view()
            .heightAnchor()
            .constraintEqualToAnchor(&native.heightAnchor())
            .setActive(true);
        Self { ui: *ui, native, rows }
    }

    pub fn row(self, row: impl NativeView) -> Self {
        if !self.rows.ns_stack_view().arrangedSubviews().is_empty() {
            self.rows.add(Divider::new(&self.ui));
        }
        self.rows.add(
            VStack::new(&self.ui)
                .insets(Insets {
                    top: 8.0,
                    left: 12.0,
                    bottom: 8.0,
                    right: 12.0,
                })
                .push(row),
        );
        self
    }
}
impl NativeView for SettingsGroup {
    fn ns_view(&self) -> &NSView { &self.native }
}

pub struct Section {
    ui: Ui,
    stack: VStack,
    group: Option<SettingsGroup>,
}
impl Section {
    pub fn new(ui: &Ui, title: &str) -> Self {
        let mut stack = VStack::new(ui).spacing(6.0);
        if !title.is_empty() {
            stack = stack.push(SectionTitle::new(ui, title));
        }
        Self { ui: *ui, stack, group: None }
    }

    pub fn subsection(ui: &Ui, title: &str) -> Self {
        Self::new(ui, "").content(SubsectionTitle::new(ui, title))
    }

    pub fn description(self, text: &str) -> Self {
        self.stack.add(SecondaryLabel::new(&self.ui, text));
        self
    }

    pub fn row(mut self, row: impl NativeView) -> Self {
        let group = self.group.take().unwrap_or_else(|| {
            let group = SettingsGroup::new(&self.ui);
            self.stack.ns_stack_view().addArrangedSubview(group.ns_view());
            group
                .ns_view()
                .widthAnchor()
                .constraintEqualToAnchor(&self.stack.ns_view().widthAnchor())
                .setActive(true);
            group
        });
        self.group = Some(group.row(row));
        self
    }

    pub fn footer(self, content: impl NativeView) -> Self {
        self.stack.add(content);
        self
    }

    pub fn content(self, content: impl NativeView) -> Self {
        self.stack.add(content);
        self
    }
}
impl NativeView for Section {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}

pub struct SettingsPage {
    ui: Ui,
    scroll: ScrollView,
    content: VStack,
}
impl SettingsPage {
    pub fn new(ui: &Ui, title: &str) -> Self {
        let content = VStack::new(ui).spacing(Metrics::SECTION_SPACING).insets(Insets {
            top: Metrics::PAGE_TOP_INSET,
            left: Metrics::PAGE_INSET,
            bottom: Metrics::PAGE_INSET,
            right: Metrics::PAGE_INSET,
        });
        if !title.is_empty() {
            content.add(Title::new(ui, title));
        }
        let outer = View::flipped(ui);
        outer.ns_view().addSubview(content.ns_view());
        crate::view::prepare(content.ns_view());
        let parent = outer.ns_view();
        let child = content.ns_view();
        child.topAnchor().constraintEqualToAnchor(&parent.topAnchor()).setActive(true);
        child
            .bottomAnchor()
            .constraintEqualToAnchor(&parent.bottomAnchor())
            .setActive(true);
        child
            .centerXAnchor()
            .constraintEqualToAnchor(&parent.centerXAnchor())
            .setActive(true);
        child
            .widthAnchor()
            .constraintLessThanOrEqualToAnchor(&parent.widthAnchor())
            .setActive(true);
        child.widthAnchor().constraintLessThanOrEqualToConstant(620.0).setActive(true);
        let fill = child.widthAnchor().constraintEqualToAnchor(&parent.widthAnchor());
        fill.setPriority(750.0);
        fill.setActive(true);
        let scroll = ScrollView::new(ui, outer);
        scroll.fit_width();
        Self { ui: *ui, scroll, content }
    }

    pub fn subtitle(self, text: &str) -> Self {
        self.content.add(SecondaryLabel::new(&self.ui, text));
        self
    }

    pub fn section(self, section: impl NativeView) -> Self {
        self.content.add(section);
        self
    }

    pub fn ns_scroll_view(&self) -> &objc2_app_kit::NSScrollView { self.scroll.ns_scroll_view() }
}
impl NativeView for SettingsPage {
    fn ns_view(&self) -> &NSView { self.scroll.ns_view() }
}

pub struct ResettableRow {
    stack: HStack,
    reset: Button,
    row: Box<dyn NativeView>,
}
impl ResettableRow {
    pub fn new(ui: &Ui, row: impl NativeView) -> Self {
        let reset = Button::new(ui, "Reset");
        let stack = HStack::new(ui);
        stack.ns_stack_view().addArrangedSubview(row.ns_view());
        stack.ns_stack_view().addArrangedSubview(reset.ns_view());
        Self {
            stack,
            reset,
            row: Box::new(row),
        }
    }

    pub fn on_reset(mut self, f: impl FnMut() + 'static) -> Self {
        self.reset = self.reset.on_click(f);
        self
    }

    pub fn set_reset_enabled(&self, value: bool) { self.reset.set_enabled(value); }

    pub fn row_view(&self) -> &NSView { self.row.ns_view() }
}
impl NativeView for ResettableRow {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}
pub struct OverrideRow {
    stack: VStack,
    toggle: Checkbox,
    row: Box<dyn NativeView>,
    inherited: SecondaryLabel,
}
impl OverrideRow {
    pub fn new(ui: &Ui, row: impl NativeView, inherited: &str) -> Self {
        let label = SecondaryLabel::new(ui, inherited);
        let weak = objc2::rc::Weak::new(row.ns_view());
        row.set_hidden(true);
        let inherited_label = objc2::rc::Weak::new(label.ns_view());
        let toggle = Checkbox::new(ui, "Use custom value").on_change(move |v| {
            if let Some(row) = weak.load() {
                row.setHidden(!v);
            }
            if let Some(label) = inherited_label.load() {
                label.setHidden(v);
            }
        });
        let stack = VStack::new(ui);
        for view in [toggle.ns_view(), label.ns_view(), row.ns_view()] {
            stack.ns_stack_view().addArrangedSubview(view);
        }
        Self {
            stack,
            toggle,
            row: Box::new(row),
            inherited: label,
        }
    }

    pub fn set_overridden(&self, value: bool) {
        self.toggle.set_value(value);
        self.row.set_hidden(!value);
        self.inherited.set_hidden(value);
    }

    pub fn set_inherited_text(&self, text: &str) { self.inherited.set_text(text); }

    pub fn on_change(mut self, mut f: impl FnMut(bool) + 'static) -> Self {
        let weak = objc2::rc::Weak::new(self.row.ns_view());
        let label = objc2::rc::Weak::new(self.inherited.ns_view());
        self.toggle = self.toggle.on_change(move |v| {
            if let Some(row) = weak.load() {
                row.setHidden(!v);
            }
            if let Some(label) = label.load() {
                label.setHidden(v);
            }
            f(v);
        });
        self
    }
}
impl NativeView for OverrideRow {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}

pub struct Validated<T: NativeControl> {
    stack: VStack,
    input: T,
    message: ValidationMessage,
}
impl<T: NativeControl> Validated<T> {
    pub fn new(ui: &Ui, input: T) -> Self {
        let stack = VStack::new(ui).spacing(4.0);
        let message = ValidationMessage::new(ui);
        stack.ns_stack_view().addArrangedSubview(input.ns_view());
        stack.ns_stack_view().addArrangedSubview(message.ns_view());
        Self { stack, input, message }
    }

    pub fn input(&self) -> &T { &self.input }

    pub fn set_validation(&self, value: &Validation) { self.message.set_validation(value); }
}
impl<T: NativeControl> NativeView for Validated<T> {
    fn ns_view(&self) -> &NSView { self.stack.ns_view() }
}
impl<T: NativeControl> NativeControl for Validated<T> {
    fn ns_control(&self) -> &objc2_app_kit::NSControl { self.input.ns_control() }
}
