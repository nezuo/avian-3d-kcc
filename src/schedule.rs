use std::time::Duration;

use bevy::{
    app::{MainScheduleOrder, RunFixedMainLoop},
    ecs::schedule::ScheduleLabel,
    gizmos::{
        clear_gizmo_context, collect_requested_gizmos, end_gizmo_context,
        gizmos::{GizmoStorage, Swap},
        propagate_gizmos, start_gizmo_context, UpdateGizmoMeshes,
    },
    prelude::*,
};

use crate::CustomStepping;

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        let mut main_schedule_order = app.world_mut().resource_mut::<MainScheduleOrder>();

        main_schedule_order.insert_after(RunFixedMainLoop, RunCustomSchedule);

        app.add_plugins(ClearCustomGizmoContextPlugin)
            .init_schedule(RunCustomSchedule)
            .init_resource::<Time<CustomTime>>()
            .add_systems(CustomMain, run_custom_main)
            .add_systems(RunCustomSchedule, run_custom_schedule);
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RunCustomSchedule;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct CustomMain;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomFirst;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomPreUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomPostUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomLast;

struct CustomTime {
    overstep: Duration,
}

impl Default for CustomTime {
    fn default() -> Self {
        Self {
            overstep: Duration::ZERO,
        }
    }
}

fn run_custom_main(world: &mut World) {
    let _ = world.try_run_schedule(CustomFirst);
    let _ = world.try_run_schedule(CustomPreUpdate);
    let _ = world.try_run_schedule(CustomUpdate);
    let _ = world.try_run_schedule(CustomPostUpdate);
    let _ = world.try_run_schedule(CustomLast);
}

fn expend_custom(a: &mut Time<CustomTime>) -> bool {
    let timestep = Duration::from_micros(15625);
    if let Some(new_value) = a.context_mut().overstep.checked_sub(timestep) {
        // reduce accumulated and increase elapsed by period
        a.context_mut().overstep = new_value;
        a.advance_by(timestep);
        true
    } else {
        // no more periods left in accumulated
        false
    }
}

pub fn run_custom_schedule(world: &mut World) {
    let custom_stepping = world.resource::<CustomStepping>();

    if custom_stepping.enabled {
        return;
    }

    let delta = world.resource::<Time<Virtual>>().delta();

    world
        .resource_mut::<Time<CustomTime>>()
        .context_mut()
        .overstep += delta;

    let _ = world.try_schedule_scope(CustomMain, |world, schedule| {
        while expend_custom(&mut world.resource_mut::<Time<CustomTime>>()) {
            *world.resource_mut::<Time>() = world.resource::<Time<CustomTime>>().as_generic();
            schedule.run(world);
        }
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}

pub fn step_custom_schedule(world: &mut World) {
    let _ = world.try_schedule_scope(CustomMain, |world, schedule| {
        *world.resource_mut::<Time>() = world.resource::<Time<CustomTime>>().as_generic();
        schedule.run(world);
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}

struct CustomGizmoContext;

struct ClearCustomGizmoContextPlugin;

impl Plugin for ClearCustomGizmoContextPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GizmoStorage<DefaultGizmoConfigGroup, CustomGizmoContext>>()
            .init_resource::<GizmoStorage<DefaultGizmoConfigGroup, Swap<CustomGizmoContext>>>()
            .add_systems(
                RunCustomSchedule,
                start_gizmo_context::<DefaultGizmoConfigGroup, CustomGizmoContext>
                    .before(run_custom_schedule),
            )
            .add_systems(
                CustomFirst,
                clear_gizmo_context::<DefaultGizmoConfigGroup, CustomGizmoContext>,
            )
            .add_systems(
                CustomLast,
                collect_requested_gizmos::<DefaultGizmoConfigGroup, CustomGizmoContext>,
            )
            .add_systems(
                RunCustomSchedule,
                end_gizmo_context::<DefaultGizmoConfigGroup, CustomGizmoContext>
                    .after(run_custom_schedule),
            )
            .add_systems(
                Last,
                propagate_gizmos::<DefaultGizmoConfigGroup, CustomGizmoContext>
                    .before(UpdateGizmoMeshes),
            );
    }
}
