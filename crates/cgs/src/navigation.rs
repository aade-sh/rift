use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::*;
use objc2_foundation::{NSArray, NSString};

use crate::*;

pub struct SplitView {
    native: Retained<NSSplitViewController>,
    view: Retained<NSView>,
    children: Vec<ViewController>,
}
impl SplitView {
    pub fn new(ui: &Ui) -> Self {
        let native = NSSplitViewController::new(ui.mtm());
        let view = native.view();
        Self {
            native,
            view,
            children: Vec::new(),
        }
    }

    pub fn pane(mut self, ui: &Ui, content: impl NativeView, sidebar: bool) -> Self {
        let controller = ViewController::new(ui, content);
        let item = if sidebar {
            NSSplitViewItem::sidebarWithViewController(controller.ns_view_controller())
        } else {
            NSSplitViewItem::splitViewItemWithViewController(controller.ns_view_controller())
        };
        if sidebar {
            item.setMinimumThickness(Metrics::SIDEBAR_MIN_WIDTH);
            item.setMaximumThickness(300.0);
            item.setCanCollapse(true);
        } else {
            item.setMinimumThickness(Metrics::DETAIL_MIN_WIDTH);
        }
        self.native.addSplitViewItem(&item);
        self.children.push(controller);
        self
    }

    pub fn ns_split_view_controller(&self) -> &NSSplitViewController { &self.native }
}
impl NativeView for SplitView {
    fn ns_view(&self) -> &NSView { &self.view }

    fn view_controller(&self) -> Option<&NSViewController> { Some(&self.native) }
}
#[derive(Clone)]
pub struct SidebarItem<T> {
    pub id: T,
    pub title: String,
    pub symbol: String,
}
pub struct Sidebar<T: 'static> {
    table: Table<SidebarItem<T>>,
}
impl<T: Clone + 'static> Sidebar<T> {
    pub fn new(ui: &Ui, items: Vec<SidebarItem<T>>) -> Self {
        let ui_copy = *ui;
        let table = Table::<SidebarItem<T>>::new(ui)
            .column("page", "", Metrics::SIDEBAR_WIDTH)
            .cells(move |item, _| {
                let row = HStack::new(&ui_copy);
                let row = if let Some(image) = ImageView::symbol(&ui_copy, &item.symbol) {
                    row.push(image)
                } else {
                    row
                };
                Box::new(row.push(Label::new(&ui_copy, &item.title)))
            })
            .rows(items);
        table.ns_table_view().setHeaderView(None);
        table.ns_table_view().setStyle(NSTableViewStyle::SourceList);
        Self { table }
    }

    pub fn on_select(mut self, mut f: impl FnMut(T) + 'static) -> Self {
        self.table = self.table.on_select_item(move |item| {
            if let Some(item) = item {
                f(item.id);
            }
        });
        self
    }

    pub fn set_selected(&self, index: usize) { self.table.set_selected(Some(index)); }

    pub fn ns_table_view(&self) -> &NSTableView { self.table.ns_table_view() }
}
impl<T: 'static> NativeView for Sidebar<T> {
    fn ns_view(&self) -> &NSView { self.table.ns_view() }
}
pub struct NavigationSplitView(SplitView);
impl NavigationSplitView {
    pub fn new(ui: &Ui, sidebar: impl NativeView, detail: impl NativeView) -> Self {
        Self(SplitView::new(ui).pane(ui, sidebar, true).pane(ui, detail, false))
    }

    pub fn ns_split_view_controller(&self) -> &NSSplitViewController {
        self.0.ns_split_view_controller()
    }
}
impl NativeView for NavigationSplitView {
    fn ns_view(&self) -> &NSView { self.0.ns_view() }

    fn view_controller(&self) -> Option<&NSViewController> { self.0.view_controller() }
}
pub struct MasterDetail(SplitView);
impl MasterDetail {
    pub fn new(ui: &Ui, master: impl NativeView, detail: impl NativeView) -> Self {
        Self(SplitView::new(ui).pane(ui, master, false).pane(ui, detail, false))
    }

    pub fn ns_split_view_controller(&self) -> &NSSplitViewController {
        self.0.ns_split_view_controller()
    }
}
impl NativeView for MasterDetail {
    fn ns_view(&self) -> &NSView { self.0.ns_view() }

    fn view_controller(&self) -> Option<&NSViewController> { self.0.view_controller() }
}

pub struct Toolbar(Retained<NSToolbar>);
impl Toolbar {
    pub fn new(ui: &Ui, id: &str) -> Self {
        Self(NSToolbar::initWithIdentifier(
            NSToolbar::alloc(ui.mtm()),
            &NSString::from_str(id),
        ))
    }

    pub fn navigation(ui: &Ui, id: &str) -> Self {
        let toolbar = Self::new(ui, id);
        toolbar.0.setItemIdentifiers(&NSArray::from_slice(&[
            unsafe { NSToolbarToggleSidebarItemIdentifier },
            unsafe { NSToolbarFlexibleSpaceItemIdentifier },
        ]));
        toolbar
    }

    pub fn attach(&self, window: &NSWindow) { window.setToolbar(Some(&self.0)); }

    pub fn ns_toolbar(&self) -> &NSToolbar { &self.0 }
}
