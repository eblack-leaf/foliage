# The host

Four things an app can ask for are things the engine cannot do itself: put text on the system
clipboard, read it back, go to a URL, and save one. Each is a verb on `Grow` like any other write,
and each is answered by whatever the platform offers.

| Verb | Does |
|---|---|
| `copy(text)` | puts text on the system clipboard |
| `paste()` | asks what the clipboard holds; the answer is `pollen.pasted()`, in a later frame |
| `navigate(url)` | goes to a URL. **On the web this replaces the page**; elsewhere the desktop is asked to open it |
| `download(url)` | asks the browser to save what is at a URL. The web's only; elsewhere it is traced and nothing else |

A `TextInput` or `TextArea` answers `Ctrl+C`, `Ctrl+X` and `Ctrl+V` for itself and does not go
through these. They are for everything that is not a field: a code block, a share link, a row of a
table.

## A share panel

```rust,no_run
use foliage::{
    Area, Boxed, Foliage, FontSize, Grove, Grow, Leaf, Location, Palette, Panel, Place, Pollen,
    Root, Rounding, Source, Text, center_x, center_y, content, left, top,
};

const LINK: &str = "https://eblack-leaf.github.io/foliage/";

struct Share {
    copy: Leaf,
    open: Leaf,
    paste: Leaf,
    shown: Leaf,
}

fn button(grove: &mut Grove, under: Leaf, y: f32, says: &str) -> Leaf {
    let button = grove.branch(
        under,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Rounding::Sm)
            .interactive()
            .at(Location::new().xs(left(16.px()).width(160.px()), top(y.px()).height(36.px()))),
    );
    grove.branch(
        button,
        Text::new(says)
            .font_size(FontSize::new().xs(14))
            .intangible()
            .at(Location::new().xs(
                center_x(50.pct()).width(content()),
                center_y(50.pct()).height(content()),
            )),
    );
    button
}

impl Root for Share {
    fn take_root(grove: &mut Grove) -> Self {
        let page = grove.plant(Panel::new());
        let copy = button(grove, page, 16.0, "copy the link");
        let open = button(grove, page, 60.0, "open it");
        let paste = button(grove, page, 104.0, "paste");
        let shown = grove.branch(
            page,
            Text::new("")
                .color(Palette::Ink.recede())
                .font_size(FontSize::new().xs(13))
                .at(Location::new().xs(
                    left(16.px()).right(100.pct() - 16.px()),
                    top(152.px()).height(content()),
                )),
        );
        Share {
            copy,
            open,
            paste,
            shown,
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.copy) {
            grove.copy(LINK);
        }
        if pollen.clicked(self.open) {
            grove.navigate(LINK);
        }
        if pollen.clicked(self.paste) {
            grove.paste();
        }
        // Never in the frame that asked.
        if let Some(text) = pollen.pasted() {
            grove.text(self.shown, format!("the clipboard held: {text}"));
        }
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.title("share");
    foliage.desktop_size(Area::new(320.0, 240.0));
    foliage.root::<Share>();
    foliage.photosynthesize();
}
```

## Pasting takes a round trip

Reading a clipboard is not instant on any platform. On the web it is a promise, and on a desktop it
is a round trip to whichever program owns the selection. So `paste` is a request, and what comes
back arrives the way bytes from a path do: as an op, drained in a later frame and reported as
`pollen.pasted()`.

An empty answer is what an app gets for an empty clipboard, and also for one the host would not
let it read: a browser asked outside a user gesture, or a desktop with no display server. There is
one thing an app can do about either, so they are one outcome, and the reason is written to the
trace.

The engine keeps a mirror of what the app itself last copied, and the mirror answers wherever the
host will not. So a copy and a paste inside the same app always round-trip, even on a host that
refuses to be read. On Android the mirror is currently the whole clipboard: the system clipboard is
not wired up there yet, so a copy reaches the app's own pastes and no further.

A field's own `Ctrl+V` takes a different road on the web: the browser hands pasted text to the
keystroke that pasted it and refuses the same read a moment later, so the engine takes the text
from the paste event itself. Either way the field is written into and reports `edited`.

## Going somewhere

`navigate` on the web sets the page's location, which replaces the page. That is deliberate: a new
tab is blocked outside a user gesture, and a frame is not one. Off the web, the desktop is asked to
open the URL with whatever it opens URLs with; on Android that is an `ACTION_VIEW` intent.

`download` is the web's alone, because a browser is what turns a URL into a file in someone's
downloads. Everywhere else it is traced and does nothing, so an app that offers a download writes
it once and it means what it can on each platform.

None of the four reaches the host under the headless test suite, which runs frames without ever
opening the platform's edges: a test never touches the clipboard or the browser of whoever runs it.
