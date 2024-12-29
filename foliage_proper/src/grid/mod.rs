#[test]
fn behavior() {
    let root_grid = Grid::new(12, 8.px()).gap((8, 4));
    let about = Location::new()
        .sm(1.to(12).pad(8), 1.to(65.vh()).min(400.px()).pad((8, 0)))
        .md(1.to(12).max(600.px()).near(), 1.to(65.vh()).min(400.px()))
        .lg(2.to(6).max(600.px()).far(), 1.to(90.vh()).max(800.px()));
    let about_grid = Grid::new(1, 8.px());
    let about_img = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct())); // cropped
    let jim = Location::new().sm(1.to(1), 14.to(external())); // font-size by height of this?
    let black = Location::new().sm(1.to(1), stem().bottom().to(external()).pad(8)); // stem(jim)
    let rva_artist = Location::new().sm(1.to(1), stem().bottom().to(external()).pad(8)); // stem(black)
    let gallery = Location::new()
        .sm(1.to(12), 65.vh().to(100.vh()).min(400.px()))
        .md(1.to(12).max(600.px()).center(), 65.vh().to(100.vh()))
        .lg(7.to(12), 0.vh().to(100.vh()));
    let gallery_grid = Grid::new(3, 1).lg(8, 8);
    let canon = Location::new().sm(1.to(1), 1.to(1)).lg(1.to(4), 2.to(6));
    let gallery_element_grid = Grid::new(1, 5).lg(5, 1);
    let canon_img = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct())); // cropped
    let c = Location::new().sm(1.to(1), 1.to(1)).lg(1.to(1), 1.to(1));
    let a = Location::new().sm(1.to(1), 2.to(2)).lg(2.to(2), 1.to(1));
    let n = Location::new().sm(1.to(1), 3.to(3)).lg(3.to(3), 1.to(1));
    let o = Location::new().sm(1.to(1), 4.to(4)).lg(4.to(4), 1.to(1));
    let n = Location::new().sm(1.to(1), 5.to(5)).lg(5.to(5), 1.to(1));
    let experimental = Location::new().sm(2.to(2), 1.to(1)).lg(4.to(6), 4.to(6));
    let experimental_img = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct()));
    let archives = Location::new().sm(3.to(3), 1.to(1)).lg(4.to(7), 6.to(8));
    let archives_img = Location::new().sm(0.pct().to(100.pct()), 0.pct().to(100.pct()));
}
