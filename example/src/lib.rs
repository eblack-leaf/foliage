use foliage::bevy_ecs::bundle::Bundle;

use foliage::bevy_ecs::event::EventWriter;
use foliage::bevy_ecs::prelude::{Commands, Entity, Resource};
use foliage::bevy_ecs::system::{ResMut, SystemParamItem};
use foliage::button::{Button, ButtonArgs, ButtonStyle};
use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, Segment, SegmentDesc};
use foliage::compositor::workflow::{
    TransitionBindValidity, TransitionDescriptor, WorkflowDescriptor, WorkflowHandle,
    WorkflowStage, WorkflowTransition,
};
use foliage::compositor::Compositor;

use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::scene::align::{SceneAligner, SceneAnchor};
use foliage::scene::bind::SceneBinder;
use foliage::scene::Scene;
use foliage::text::{MaxCharacters, TextValue};
use foliage::window::WindowDescriptor;
use foliage::{bevy_ecs, scene_bind_enable};
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((360, 800)),
        )
        .with_leaf::<Tester>()
        .with_android_interface(android_interface)
        .run();
}
#[derive(Resource)]
struct ToDespawn(Entity);
struct Tester;
fn spawn_button_tree(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut events: EventWriter<WorkflowTransition>,
) {
    let segment_one_handle =
        compositor.add_segment(ResponsiveSegment::mobile_portrait(Segment::new(
            (0.085.relative(), 0.11.relative()),
            (0.83.relative(), 0.11.relative()),
            4,
        )));
    let segment_two_handle = compositor.add_segment(ResponsiveSegment::mobile_portrait(
        Segment::new((85.fixed(), 250.fixed()), (240.fixed(), 75.fixed()), 4),
    ));
    let segment_three_handle = compositor.add_segment(ResponsiveSegment::mobile_portrait(
        Segment::new((140.fixed(), 375.fixed()), (135.fixed(), 50.fixed()), 4),
    ));
    let segment_four_handle = compositor.add_segment(ResponsiveSegment::mobile_portrait(
        Segment::new((35.fixed(), 700.fixed()), (340.fixed(), 50.fixed()), 4),
    ));
    let transition = TransitionDescriptor::new(&mut cmd)
        .bind_scene::<Button>(vec![
            (
                segment_one_handle,
                TransitionBindValidity::all(),
                ButtonArgs::new(
                    ButtonStyle::Ring,
                    TextValue::new("Afternoon"),
                    MaxCharacters(9),
                    IconId::new(BundledIcon::Umbrella),
                    Color::RED.into(),
                    Color::OFF_BLACK.into(),
                ),
            ),
            (
                segment_two_handle,
                TransitionBindValidity::all(),
                ButtonArgs::new(
                    ButtonStyle::Ring,
                    TextValue::new("Fore-"),
                    MaxCharacters(5),
                    IconId::new(BundledIcon::Droplet),
                    Color::GREEN.into(),
                    Color::OFF_BLACK.into(),
                ),
            ),
            (
                segment_three_handle,
                TransitionBindValidity::all(),
                ButtonArgs::new(
                    ButtonStyle::Ring,
                    TextValue::new("CAST!"),
                    MaxCharacters(5),
                    IconId::new(BundledIcon::Cast),
                    Color::BLUE.into(),
                    Color::OFF_BLACK.into(),
                ),
            ),
        ])
        .bind_scene::<DualButton>(vec![(
            segment_four_handle,
            TransitionBindValidity::all(),
            ButtonArgs::new(
                ButtonStyle::Ring,
                TextValue::new("Rainy-Day"),
                MaxCharacters(9),
                IconId::new(BundledIcon::CloudDrizzle),
                Color::CYAN_MEDIUM.into(),
                Color::CYAN_DARK.into(),
            ),
        )])
        .build();
    compositor.add_workflow(
        WorkflowDescriptor::new(WorkflowHandle(0))
            .with_transition(WorkflowStage(0), transition)
            .workflow(),
    );
    events.send(WorkflowTransition(WorkflowHandle(0), WorkflowStage(0)));
}
impl Leaf for Tester {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((spawn_button_tree,));
        scene_bind_enable!(elm, Button, DualButton);
    }
}
#[derive(Bundle)]
struct DualButton {}
impl Scene for DualButton {
    type Args<'a> = <Button as Scene>::Args<'a>;
    type ExternalResources = <Button as Scene>::ExternalResources;
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalResources>,
        binder: &mut SceneBinder,
    ) -> Self {
        binder.bind_scene::<Button>(
            0.into(),
            ((-5).near(), 0.near(), 0).into(),
            anchor.0.section.area / (2, 1).into(),
            args,
            external_args,
            cmd,
        );
        binder.bind_scene::<Button>(
            1.into(),
            ((-5).far(), 0.near(), 0).into(),
            anchor.0.section.area / (2, 1).into(),
            args,
            external_args,
            cmd,
        );
        Self {}
    }
}
