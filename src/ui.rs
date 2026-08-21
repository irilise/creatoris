use crate::bodies::{CelestialBody, CelestialType, OrbitParent, SolarSystemDirectory};
use crate::orbital_math::OrbitalElements;
use crate::sim_clock::SimClock;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, update_hud_text);
    }
}

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct TimeWarpText;

#[derive(Component)]
struct TargetInfoText;

fn setup_hud(mut commands: Commands) {
    // Root container (full screen overlay)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // Top Bar: Simulation Status & Clock
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.85)),
                BorderColor::all(Color::srgba(0.2, 0.35, 0.6, 0.4)),
            ))
            .with_children(|top_bar| {
                // Left: Title and Play/Pause status badge
                top_bar
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(14.0),
                            ..default()
                        },
                    ))
                    .with_children(|left| {
                        left.spawn((
                            Text::new("CREATORIS // SOLAR SYSTEM"),
                            TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.75, 1.0)),
                        ));

                        left.spawn((
                            StatusText,
                            Text::new("▶ PLAYING [SPACE]"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.2, 0.9, 0.4)),
                        ));
                    });

                // Right: Time scale and elapsed simulation time
                top_bar.spawn((
                    TimeWarpText,
                    Text::new("Warp: 500,000x | Year 0.00 (Day 0.0)"),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.9, 0.98)),
                ));
            });

            // Bottom Section
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexEnd,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
            ))
            .with_children(|bottom_row| {
                // Left Card: Focused Body Telemetry
                bottom_row
                    .spawn((
                        Node {
                            width: Val::Px(360.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.88)),
                        BorderColor::all(Color::srgba(0.25, 0.4, 0.65, 0.45)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            TargetInfoText,
                            Text::new("Loading celestial telemetry..."),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.93, 0.98)),
                        ));
                    });

                // Right Card: Quick Controls Reference
                bottom_row
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.88)),
                        BorderColor::all(Color::srgba(0.25, 0.4, 0.65, 0.45)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new("NAVIGATION & CONTROLS"),
                            TextFont {
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.4, 0.7, 1.0)),
                        ));
                        card.spawn((
                            Text::new("[SPACE] Play / Pause\n[0-9 / P / Tab] Select Body (0:Sun 1:Mer 2:Ven 3:Earth 4:Moon 5:Mars 6:Jup 7:Sat 8:Ura 9:Nep)\n[Drag / WASD / Arrows] Orbit View | [Scroll / Q / E] Zoom\n[[ / ]] Warp Speed | [Z-B] Warp Presets (1x - 10Mx)\n[O] Toggle Orbits | [I] Toggle Asteroid Orbits"),
                            TextFont {
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(Color::srgb(0.72, 0.78, 0.88)),
                        ));
                    });
            });
        });
}

fn update_hud_text(
    clock: Res<SimClock>,
    directory: Res<SolarSystemDirectory>,
    mut status_query: Query<(&mut Text, &mut TextColor), (With<StatusText>, Without<TimeWarpText>, Without<TargetInfoText>)>,
    mut time_query: Query<&mut Text, (With<TimeWarpText>, Without<StatusText>, Without<TargetInfoText>)>,
    mut target_query: Query<&mut Text, (With<TargetInfoText>, Without<StatusText>, Without<TimeWarpText>)>,
    body_query: Query<(
        &CelestialBody,
        Option<&OrbitalElements>,
        Option<&OrbitParent>,
    )>,
) {
    // 1. Update Play/Pause status
    for (mut text, mut color) in status_query.iter_mut() {
        if clock.paused {
            *text = Text::new("⏸ PAUSED [SPACE]");
            color.0 = Color::srgb(1.0, 0.7, 0.2); // Warm Amber
        } else {
            *text = Text::new("▶ PLAYING [SPACE]");
            color.0 = Color::srgb(0.2, 0.9, 0.45); // Emerald Green
        }
    }

    // 2. Update Clock and Warp
    for mut text in time_query.iter_mut() {
        let warp_str = if clock.time_scale >= 1_000_000.0 {
            format!("{:.1}M x", clock.time_scale / 1_000_000.0)
        } else if clock.time_scale >= 1_000.0 {
            format!("{:.0}k x", clock.time_scale / 1_000.0)
        } else {
            format!("{:.0} x", clock.time_scale)
        };

        let years = clock.elapsed_years();
        let days = clock.elapsed_days();

        *text = Text::new(format!(
            "Warp: {} | Time: Year {:.2} (Day {:.1})",
            warp_str, years, days
        ));
    }

    // 3. Update Target Telemetry Card
    for mut text in target_query.iter_mut() {
        if let Some(target_entity) = directory.current_entity() {
            if let Ok((body, elements, parent)) = body_query.get(target_entity) {
                let type_name = match body.celestial_type {
                    CelestialType::Star => "Central Star",
                    CelestialType::Planet => "Major Planet",
                    CelestialType::DwarfPlanet => "Dwarf Planet",
                    CelestialType::Moon => "Natural Satellite",
                    CelestialType::Asteroid => "Asteroid",
                };

                let orbit_info = if let Some(elem) = elements {
                    if parent.is_some() {
                        format!(
                            "Orbit Distance: {:.0} km (Local)\nOrbital Period: {:.2} Days",
                            elem.semi_major_axis,
                            elem.period_days()
                        )
                    } else {
                        format!(
                            "Semi-Major Axis: {:.3} AU\nOrbital Period: {:.2} Earth Years ({:.1} Days)\nEccentricity: {:.4} | Inclination: {:.2}°",
                            elem.semi_major_axis_au(),
                            elem.period_years(),
                            elem.period_days(),
                            elem.eccentricity,
                            elem.inclination.to_degrees()
                        )
                    }
                } else {
                    "Coordinates: System Origin (0, 0, 0)\nGravitational Center".to_string()
                };

                *text = Text::new(format!(
                    "TARGET: {}\nType: {}\nMass: {:.3e} kg | Radius: {:.0} km\n{}\n\n{}",
                    body.name.to_uppercase(),
                    type_name,
                    body.mass,
                    body.radius,
                    orbit_info,
                    body.description
                ));
            }
        }
    }
}
