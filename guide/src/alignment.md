# Alignment

An `Alignment` is used to place an element in a `Scene`.

Here is the calculation
```rust
impl PositionAlignment {
    pub fn calc_pos(
        &self,
        anchor: Anchor,
        node_area: Area<InterfaceContext>,
    ) -> Position<InterfaceContext> {
        let x = match self.horizontal.bias {
            AlignmentBias::Near => anchor.0.section.left() + self.horizontal.offset,
            AlignmentBias::Center => {
                anchor.0.section.center().x - node_area.width / 2f32 + self.horizontal.offset
            }
            AlignmentBias::Far => {
                anchor.0.section.right() - self.horizontal.offset - node_area.width
            }
        };
        let y = match self.vertical.bias {
            AlignmentBias::Near => anchor.0.section.top() + self.vertical.offset,
            AlignmentBias::Center => {
                anchor.0.section.center().y - node_area.height / 2f32 + self.vertical.offset
            }
            AlignmentBias::Far => {
                anchor.0.section.bottom() - self.vertical.offset - node_area.height
            }
        };
        (x, y).into()
    }
}
```

It matches the `Bias` of `Near/Center/Far` in each direction to snap to relative offsets from that
location. `Near` starts at the `left-x`, `Center` uses the `width / 2`, and `Far` starts from
the `.right()` of the `Anchor`.