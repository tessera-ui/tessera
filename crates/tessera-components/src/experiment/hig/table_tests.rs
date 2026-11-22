use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, Modifier, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::column::column;

use super::{super::text::text, TableColumn, TableDisclosure, TableSort, table};

#[derive(Clone, Copy)]
struct Controls {
    selection: State<Option<usize>>,
    sort: State<TableSort>,
}

const FILES: [(&str, &str); 3] = [("b.txt", "2 KB"), ("a.txt", "9 KB"), ("c.txt", "1 KB")];

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let selection = remember(|| None);
    let sort = remember(|| TableSort {
        column: 0,
        ascending: true,
    });
    probe.set(Some(Controls { selection, sort }));
    let mut rows = FILES.to_vec();
    let current = sort.get();
    rows.sort_by_key(|(name, size)| if current.column == 0 { *name } else { *size });
    if !current.ascending {
        rows.reverse();
    }
    column().children(move || {
        let rows = rows.clone();
        table(
            vec![
                TableColumn::new("Name"),
                TableColumn::new("Size").width(Dp(60.0)),
            ],
            rows.len(),
        )
        .sort(current)
        .selection_optional(selection.get())
        .on_sort(move |next| sort.set(next))
        .on_select(move |row| selection.set(Some(row)))
        .accessibility_label("Files")
        .modifier(Modifier::new().size(Dp(260.0), Dp(200.0)))
        .cell(move |(row, column)| {
            let (name, size) = rows[row];
            text(if column == 0 { name } else { size });
        });
    });
}

fn names(ui: &UiTest) -> Vec<String> {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .filter(|(_, node)| node.role() == Role::Label)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .filter(|label| label.ends_with(".txt"))
        .collect()
}

#[test]
fn header_clicks_request_sorting_and_rows_select() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().expect("fixture state");
    assert_eq!(names(&ui), ["a.txt", "b.txt", "c.txt"]);
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    assert!(snapshot.find("Files", Role::Table).is_some());
    let (name, _) = snapshot.find("Name", Role::ColumnHeader).expect("header");
    assert!(ui.accessibility_action(name, Action::Click));
    ui.frames(2);
    assert_eq!(
        controls.sort.get(),
        TableSort {
            column: 0,
            ascending: false
        },
        "clicking the sorted column reverses it"
    );
    assert_eq!(names(&ui), ["c.txt", "b.txt", "a.txt"]);
    let rows: Vec<_> = ui
        .inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .filter(|(_, node)| node.role() == Role::Row && node.supports_action(Action::Click))
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(rows.len(), 3);
    assert!(ui.accessibility_action(rows[1], Action::Click));
    ui.frames(2);
    assert_eq!(controls.selection.get(), Some(1));
}

#[test]
fn fixed_columns_keep_their_width_and_flexible_columns_share_the_rest() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 300)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let width = |label: &str| {
        snapshot
            .find(label, Role::ColumnHeader)
            .and_then(|(_, node)| node.bounds())
            .map(|bounds| bounds.x1 - bounds.x0)
            .expect("header bounds")
    };
    assert_eq!(width("Size"), 60.0);
    assert_eq!(width("Name"), 200.0);
}

#[tessera]
fn outline_fixture(probe: TestProbe<Option<State<bool>>>) {
    let expanded = remember(|| false);
    probe.set(Some(expanded));
    let open = expanded.get();
    let mut names = vec!["Documents"];
    let mut outline = vec![TableDisclosure {
        depth: 0,
        expanded: Some(open),
    }];
    if open {
        names.push("Report.pdf");
        outline.push(TableDisclosure {
            depth: 1,
            expanded: None,
        });
    }
    names.push("Downloads");
    outline.push(TableDisclosure {
        depth: 0,
        expanded: None,
    });
    column().children(move || {
        let names = names.clone();
        table(vec![TableColumn::new("Name")], names.len())
            .disclosure(outline.clone())
            .on_expanded_change(move |(row, next)| {
                assert_eq!(row, 0);
                expanded.set(next);
            })
            .modifier(Modifier::new().size(Dp(260.0), Dp(200.0)))
            .cell(move |(row, _)| {
                text(names[row]);
            });
    });
}

fn labels(ui: &UiTest) -> Vec<String> {
    ui.inspect_accessibility()
        .expect("published semantics")
        .nodes()
        .iter()
        .filter(|(_, node)| node.role() == Role::Label)
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

#[test]
fn disclosure_rows_expand_and_indent_their_children() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 300)
        .content(|| outline_fixture(probe.clone()));
    ui.frame();
    let expanded = probe.get().expect("fixture state");
    assert_eq!(labels(&ui), ["Name", "Documents", "Downloads"]);
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (triangle, node) = snapshot
        .nodes()
        .iter()
        .find(|(_, node)| node.role() == Role::DisclosureTriangle)
        .map(|(id, node)| (*id, node.clone()))
        .expect("disclosure triangle");
    assert_eq!(node.is_expanded(), Some(false));
    assert!(ui.accessibility_action(triangle, Action::Expand));
    ui.frames(2);
    assert!(expanded.get());
    assert_eq!(
        labels(&ui),
        ["Name", "Documents", "Report.pdf", "Downloads"]
    );
    let x = |label: &str| {
        ui.inspect_accessibility()
            .expect("published semantics")
            .find(label, Role::Label)
            .and_then(|(_, node)| node.bounds())
            .expect("label bounds")
            .x0
    };
    assert_eq!(
        x("Report.pdf") - x("Documents"),
        16.0,
        "children indent one level"
    );
    assert_eq!(x("Documents"), x("Downloads"));
}
