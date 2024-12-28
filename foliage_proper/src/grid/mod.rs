#[test]
fn behavior() {
    let root = Location::new().mp(0.pct().to(100.pct()), 0.pct().to(100.pct()));
    let root_grid = Grid::new(12, repeat(16.px()));
    let about = Location::new()
        .mp(1.to(12), 1.to(75.pct()).min(400.px()).max(900.px()))
        .ml(2.to(4), 1.to(100.pct()).min(400.px())); // stem(root)
    let about_grid = Grid::new(1, 5);
    let image = Location::new().mp(1.to(1), 1.to(100.pct())); // stem(about)
    let header = Location::new().mp(1.to(1), 2.to(4)); // stem(about)
    let header_grid = Grid::new(1, repeat(16.px()));
    let first_name = Location::new().mp(1.to(1), 1.to(auto())); // stem(header) + to(auto()) => content-based-height
    let second_name = Location::new().mp(1.to(1), stem().bottom().to(auto())); // stem(first_name)
    let tagline = Location::new().mp(1.to(1), stem().bottom().to(auto())); // stem(second_name)
}
