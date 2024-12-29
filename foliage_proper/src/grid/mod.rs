#[test]
fn behavior() {
    let grid = Grid::new(12.gap(4), 8.px().gap(4))
        .md(12.gap(4), 8.px().gap(4))
        .lg(12.gap(8), 16.px().gap(8))
        .xl(12.gap(12), 24.px().gap(12)); // canon
    let root = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct()));
    // let view = View::context(root); // scrolling
    let location = Location::new().sm(x(50.px()).y(100.px()), x(50.px()).y(150.px())); // points
    let location = Location::new().sm(1.to(12), 1.to(19));
    let location = Location::new()
        .sm(
            2.to(11).max(400.px()).justify(Center).pad(4),
            4.to(10).pad((4, 8)), // debug-assert max only on width
        )
        .md(3.to(10).max(500.px()).justify(Center), 4.to(10));
    let location = Location::new()
        .sm(3.to(10).max(300.px()).justify(Left), 6.to(9))
        .md(4.to(9).max(400.px()).justify(Left), 6.to(9));
    let location = Location::new().sm(1.to(1), 2.to(auto()));
    let location = Location::new().sm(1.to(1), stack().to(auto()));// stack uses stem().bottom() as this.top()
    let location = Location::new().sm(1.to(1), stack().to(25)); // way to hook back into known row
}
