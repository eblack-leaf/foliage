use foliage::bevy_ecs::bundle::Bundle;

use foliage::bevy_ecs::event::EventWriter;
use foliage::bevy_ecs::prelude::{Commands, Entity, IntoSystemConfigs, Resource, Without};
use foliage::bevy_ecs::query::{Changed, With};
use foliage::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage::button::{Button, ButtonArgs, ButtonBindings, ButtonStyle};
use foliage::color::Color;
use foliage::compositor::segment::{ResponsiveSegment, Segment, SegmentDesc};
use foliage::compositor::workflow::{
    TransitionBindValidity, TransitionDescriptor, WorkflowDescriptor, WorkflowHandle,
    WorkflowStage, WorkflowTransition,
};
use foliage::compositor::Compositor;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::r_scene::align::SceneAligner;
use foliage::r_scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::text::{MaxCharacters, TextValue};
use foliage::window::WindowDescriptor;
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};
use foliage::{AndroidInterface, Foliage};

pub fn entry(android_interface: AndroidInterface) {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((360, 800)),
        )
        .with_leaf::<DualButton>()
        .with_android_interface(android_interface)
        .run();
}
#[derive(Resource)]
struct ToDespawn(Entity);
fn spawn_button_tree(
    mut cmd: Commands,
    mut compositor: ResMut<Compositor>,
    mut events: EventWriter<WorkflowTransition>,
) {
    let segment = Segment::new(
        (0.085.relative(), 0.11.relative()),
        (0.83.relative(), 0.11.relative()),
        4,
    );
    let segment_one_handle = compositor.add_segment(
        ResponsiveSegment::all(segment)
            .with_landscape_desktop(segment.with_area((0.25.relative(), 0.05.relative()))),
    );
    let segment_two_handle = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (85.fixed(), 250.fixed()),
        (240.fixed(), 75.fixed()),
        4,
    )));
    let segment_three_handle = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (140.fixed(), 375.fixed()),
        (135.fixed(), 50.fixed()),
        4,
    )));
    let segment_four_handle = compositor.add_segment(ResponsiveSegment::all(Segment::new(
        (0.10.relative(), 0.80.relative()),
        (0.8.relative(), 50.fixed()),
        4,
    )));
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
            // (
            //     segment_two_handle,
            //     TransitionBindValidity::all(),
            //     ButtonArgs::new(
            //         ButtonStyle::Ring,
            //         TextValue::new("Fore-"),
            //         MaxCharacters(5),
            //         IconId::new(BundledIcon::Droplet),
            //         Color::GREEN.into(),
            //         Color::OFF_BLACK.into(),
            //     ),
            // ),
            // (
            //     segment_three_handle,
            //     TransitionBindValidity::all(),
            //     ButtonArgs::new(
            //         ButtonStyle::Ring,
            //         TextValue::new("CAST!"),
            //         MaxCharacters(5),
            //         IconId::new(BundledIcon::Cast),
            //         Color::BLUE.into(),
            //         Color::OFF_BLACK.into(),
            //     ),
            // ),
        ])
        .bind_scene::<DualButton>(vec![(
            segment_four_handle,
            TransitionBindValidity::all(),
            ButtonArgs::new(
                ButtonStyle::Ring,
                TextValue::new("first-text"),
                MaxCharacters(10),
                IconId::new(BundledIcon::Clipboard),
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
set_descriptor!(
    enum SetDescriptor {
        Area,
    }
);
fn resize_dual_button(
    mut coordinator: ResMut<SceneCoordinator>,
    query: Query<
        (&Area<InterfaceContext>, &SceneHandle),
        (
            Changed<Area<InterfaceContext>>,
            With<Tag<DualButton>>,
            Without<Tag<Button>>,
        ),
    >,
    mut button_areas: Query<&mut Area<InterfaceContext>, Without<Tag<DualButton>>>,
    mut text: Query<&mut TextValue>,
) {
    for (area, handle) in query.iter() {
        let coordinate = coordinator.anchor(*handle).0.with_area(*area);
        coordinator.update_anchor(*handle, coordinate);
        let first_button =
            coordinator.binding_entity(&handle.access_chain().binding(DualButtonBindings::First));
        let half_area = *area / (2, 1).into();
        *button_areas.get_mut(first_button).unwrap() = half_area;
        let second_button =
            coordinator.binding_entity(&handle.access_chain().binding(DualButtonBindings::Second));
        *button_areas.get_mut(second_button).unwrap() = half_area;
        let text_entity = coordinator.binding_entity(
            &handle
                .access_chain()
                .binding(DualButtonBindings::Second)
                .binding(ButtonBindings::Text),
        );
        *text.get_mut(text_entity).unwrap() = TextValue::new("second-text");
    }
}
impl Leaf for DualButton {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((spawn_button_tree,));
        elm.main().add_systems((resize_dual_button
            .in_set(ExternalSet::Configure)
            .in_set(SetDescriptor::Area)
            .before(<Button as Leaf>::SetDescriptor::Button),));
        scene_bind_enable!(elm, Button, DualButton);
    }
}
#[derive(Bundle)]
struct DualButton {
    tag: Tag<DualButton>,
}
enum DualButtonBindings {
    First,
    Second,
}
impl From<DualButtonBindings> for SceneBinding {
    fn from(value: DualButtonBindings) -> Self {
        SceneBinding::from(value as i32)
    }
}
impl Scene for DualButton {
    type Bindings = DualButtonBindings;
    type Args<'a> = <Button as Scene>::Args<'a>;
    type ExternalArgs = <Button as Scene>::ExternalArgs;
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder,
    ) -> Self {
        // let transition = SceneTransitionDescriptor::new(cmd, binder.scene_transition_root())
        //     .bind_scene::<Button>(vec![(
        //         0.into(),
        //         ((-5).near(), 0.near(), 0).into(),
        //         ButtonArgs::new(
        //             args.style,
        //             TextValue::new("changed"),
        //             MaxCharacters(7),
        //             args.icon_id,
        //             args.foreground_color,
        //             args.background_color,
        //         ),
        //     )])
        //     .build();
        // cmd.entity(binder.this())
        //     .insert(
        //         SceneWorkflow::new().with_workflow(
        //             WorkflowDescriptor::new(WorkflowHandle(0))
        //                 .with_transition(WorkflowStage(0), transition)
        //                 .workflow(),
        //         ),
        //     )
        //     .insert(WorkflowTransitionQueue(vec![WorkflowTransition(
        //         WorkflowHandle(0),
        //         WorkflowStage(0),
        //     )]));
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
        Self {
            tag: Tag::<DualButton>::new(),
        }
    }
}
