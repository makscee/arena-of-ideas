//! Integration examples showing how to use the render system in real game UI scenarios

use super::*;
use crate::ui::render::composers::menu::MenuAction;
use crate::ui::see::CstrTrait;

/// Main game UI using the render system
pub fn render_game_ui(world: &World, ui: &mut Ui) {
    Context::from_world_ref_r(world, |context| {
        ui.horizontal(|ui| {
            render_sidebar(context, ui);
            ui.separator();
            render_main_content(context, ui);
        });
        Ok(())
    })
    .unwrap_or_else(|e| eprintln!("Error in render_game_ui: {:?}", e));
}

/// Sidebar with player info and navigation
fn render_sidebar(context: &Context, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.heading("Arena of Ideas");

        // Player section
        if let Ok(owner) = context.owner_entity() {
            if let Ok(player) = context.component::<NPlayer>(owner) {
                Frame::new()
                    .inner_margin(4)
                    .corner_radius(ROUNDING)
                    .stroke(subtle_borders_and_separators().stroke())
                    .show(ui, |ui| {
                        player.render(context).display(ui);

                        if let Ok(match_data) = player.active_match_load(context) {
                            ui.separator();
                            match_data.render(context).display(ui);
                        }
                    });
            }
        }

        ui.separator();

        // Navigation
        if ui.button("🏠 Home").clicked() {
            // Navigate to home
        }
        if ui.button("⚔️ Battle").clicked() {
            // Navigate to battle
        }
        if ui.button("📦 Collection").clicked() {
            // Navigate to collection
        }
        if ui.button("🏪 Shop").clicked() {
            // Navigate to shop
        }
    });
}

/// Main content area
fn render_main_content(context: &Context, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        // Match the current view state
        render_team_builder(context, ui);
    });
}

/// Team builder view
fn render_team_builder(context: &Context, ui: &mut Ui) {
    ui.heading("Team Builder");

    if let Ok(owner) = context.owner_entity() {
        if let Ok(player) = context.component::<NPlayer>(owner) {
            // Get team from active match
            if let Some(active_match) = player.active_match.get_data() {
                if let Some(team) = active_match.team.get_data() {
                    // Team overview
                    Frame::new()
                        .inner_margin(8)
                        .corner_radius(ROUNDING)
                        .fill(ui.visuals().extreme_bg_color)
                        .show(ui, |ui| {
                            team.render(context).tag(ui);
                            ui.separator();
                            team.render(context).display(ui);
                        });

                    ui.add_space(8.0);

                    // Houses section
                    ui.heading("Houses");
                    render_houses(&team, context, ui);

                    ui.add_space(8.0);

                    // Fusions section
                    ui.heading("Fusions");
                    render_fusions(&team, context, ui);
                }
            }
        }
    }

    /// Render houses in a team
    fn render_houses(team: &NTeam, context: &Context, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if let Some(houses) = team.houses.get_data() {
                for house in houses {
                    render_house_card(house, context, ui);
                }
            }

            // Add house button
            if ui.button("➕ Add House").clicked() {
                // Open house selection dialog
            }
        });
    }

    /// Render a house card with units
    fn render_house_card(house: &NHouse, context: &Context, ui: &mut Ui) {
        Frame::new()
            .inner_margin(8)
            .corner_radius(ROUNDING)
            .stroke(egui::Stroke::new(1.0, house.color_for_text(context)))
            .show(ui, |ui| {
                // House header with context menu
                let response = house
                    .render(context)
                    .with_menu()
                    .add_action("✏️ Edit".to_string(), |house, ctx| {
                        // Open edit dialog
                        None
                    })
                    .add_action("📋 Duplicate".to_string(), |house, ctx| {
                        Some(MenuAction::Custom(Box::new(house)))
                    })
                    .add_dangerous_separator()
                    .add_delete()
                    .show_with(ui, |builder, ui| builder.title_button(ui));

                if let Some(deleted) = response.deleted() {
                    // Handle deletion
                }

                ui.separator();

                // House abilities
                ui.horizontal(|ui| {
                    if let Ok(ability) = house.ability_load(context) {
                        ui.label("Ability:");
                        ability.render(context).tag(ui);
                    }
                });

                ui.horizontal(|ui| {
                    if let Ok(status) = house.status_load(context) {
                        ui.label("Status:");
                        status.render(context).tag(ui);
                    }
                });

                ui.separator();

                // Units grid
                ui.label("Units:");
                render_units_grid(&house.units, context, ui);
            });
    }

    /// Render units in a grid layout
    fn render_units_grid(units: &NodeParts<Child, NUnit>, context: &Context, ui: &mut Ui) {
        egui::Grid::new("units_grid")
            .num_columns(3)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, unit) in units.iter().enumerate() {
                    // Use draggable composer for drag and drop
                    let draggable = DraggableComposer::new(
                        TagCardComposer::default(),
                        format!("unit_drag_{}", unit.id),
                    );

                    unit.render(context).with_composer(draggable).compose(ui);

                    if (i + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }

                // Add unit button
                if ui.button("➕").on_hover_text("Add Unit").clicked() {
                    // Open unit selection
                }
            });
    }

    /// Render fusions
    fn render_fusions(team: &NTeam, context: &Context, ui: &mut Ui) {
        if let Some(fusions) = team.fusions.get_data() {
            for fusion in fusions {
                render_fusion_panel(fusion, context, ui);
            }
        }

        if ui.button("➕ Add Fusion").clicked() {
            // Create new fusion
        }
    }

    /// Render a fusion panel
    fn render_fusion_panel(fusion: &NFusion, context: &Context, ui: &mut Ui) {
        Frame::new()
            .inner_margin(8)
            .corner_radius(ROUNDING)
            .fill(Color32::from_rgb(128, 0, 128).gamma_multiply(0.1))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(128, 0, 128)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    fusion.render(context).title(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        fusion.render(context).tag(ui);
                    });
                });

                ui.separator();

                // Fusion stats
                fusion.render(context).display(ui);

                ui.separator();

                // Fusion slots
                ui.label("Slots:");
                ui.horizontal(|ui| {
                    if let Some(slots) = fusion.slots.get_data() {
                        for slot in slots {
                            render_fusion_slot(slot, fusion, context, ui);
                        }
                    }
                });
            });
    }

    /// Render a fusion slot with drop target
    fn render_fusion_slot(slot: &NFusionSlot, fusion: &NFusion, context: &Context, ui: &mut Ui) {
        let drop_target = DropTargetComposer::new(
            TagComposer,
            format!("unit_drag_{}", slot.id),
            move |unit: NUnit, ctx| {
                // Handle unit drop into slot
                println!("Unit {} dropped into slot", unit.unit_name);
            },
        );

        Frame::new()
            .inner_margin(4)
            .corner_radius(ROUNDING)
            .stroke(egui::Stroke::new(
                1.0,
                Color32::from_rgb(128, 0, 128).gamma_multiply(0.5),
            ))
            .show(ui, |ui| {
                if let Ok(unit) = slot.unit_load(context) {
                    unit.render(context).with_composer(drop_target).compose(ui);
                } else {
                    // Empty slot
                    ui.centered_and_justified(|ui| {
                        "Empty Slot"
                            .cstr_c(Color32::from_rgb(128, 128, 128))
                            .label(ui);
                    });
                }
            });
    }
}

/// Battle view using the render system
pub fn render_battle_view(context: &Context, ui: &mut Ui) {
    ui.heading("Battle");

    // Get current battle
    if let Ok(battle) = context
        .owner_entity()
        .and_then(|owner| context.component::<NBattle>(owner))
    {
        // Battle status
        battle.render(context).card(ui);

        ui.separator();

        // Team displays
        ui.columns(2, |columns| {
            // Left team
            columns[0].vertical_centered(|ui| {
                ui.heading("Your Team");
                if let Ok(team) = context.component::<NTeam>(Entity::from_bits(battle.team_left)) {
                    render_battle_team(&team, context, ui);
                }
            });

            // Right team
            columns[1].vertical_centered(|ui| {
                ui.heading("Enemy Team");
                if let Ok(team) = context.component::<NTeam>(Entity::from_bits(battle.team_right)) {
                    render_battle_team(&team, context, ui);
                }
            });
        });
    }
}

/// Render a team in battle
fn render_battle_team(team: &NTeam, context: &Context, ui: &mut Ui) {
    // Use grouped composer to organize by house
    let grouped = GroupedComposer::new(CardComposer, |unit: &NUnit, ctx| {
        // Group by parent house name
        "Default House".to_string()
    });

    // Collect all units from team
    let mut all_units = Vec::new();
    if let Some(houses) = team.houses.get_data() {
        for house in houses {
            if let Some(units) = house.units.get_data() {
                for unit in units {
                    all_units.push(unit.clone());
                }
            }
        }
    }

    all_units.render(context).with_composer(grouped).compose(ui);
}

/// Collection view with search and filters
pub fn render_collection_view(context: &Context, ui: &mut Ui) {
    ui.heading("Collection");

    // Search bar
    let mut search_query = String::new();
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut search_query);

        if ui.button("Clear").clicked() {
            search_query.clear();
        }
    });

    ui.separator();

    // Get all units in collection
    if let Ok(units) = context.collect_children_components::<NUnit>(0) {
        // Create a list composer with filtering, sorting, and pagination
        let base_composer: ListComposer<NUnit, TagCardComposer> =
            ListComposer::new(TagCardComposer::default());

        // Apply filters and sorting directly to the units vector
        let mut filtered_units: Vec<NUnit> = units
            .into_iter()
            .filter(|unit| {
                if search_query.is_empty() {
                    true
                } else {
                    unit.unit_name
                        .to_lowercase()
                        .contains(&search_query.to_lowercase())
                }
            })
            .cloned()
            .collect();

        // Sort by unit name
        filtered_units.sort_by(|a, b| a.unit_name.cmp(&b.unit_name));

        // Render the list
        ScrollArea::vertical().show(ui, |ui| {
            if filtered_units.is_empty() {
                ui.label("No units match your search");
            } else {
                // Apply pagination manually
                let items_per_page = 20;
                let page = 0; // You could add state for this
                let start = page * items_per_page;
                let end = ((page + 1) * items_per_page).min(filtered_units.len());

                for unit in &filtered_units[start..end] {
                    unit.render(context)
                        .with_composer(TagCardComposer::default())
                        .compose(ui);
                }
            }
        });
    }
}

/// Shop view with selectable items
pub fn render_shop_view(context: &Context, ui: &mut Ui) {
    ui.heading("Shop");

    if let Ok(owner) = context.owner_entity() {
        if let Ok(match_data) = context.component::<NMatch>(owner) {
            // Shop currency
            ui.horizontal(|ui| {
                ui.label("Gold:");
                format!("{}", match_data.g)
                    .cstr_c(Color32::from_rgb(255, 255, 0))
                    .label(ui);
            });

            ui.separator();

            // Shop offers
            ui.label("Available Items:");

            // TODO: Implement shop offer rendering when FTag is implemented for ShopOffer
            // and proper shop slot handling is added
            for offer in &match_data.shop_offers {
                ui.horizontal(|ui| {
                    ui.label(format!("Buy limit: {:?}", offer.buy_limit));
                    for slot in &offer.case {
                        if !slot.sold {
                            ui.label(format!("Item #{} - {} gold", slot.node_id, slot.price));
                        }
                    }
                });
            }
        }
    }
}

/// Debug view showing all features of a node
pub fn render_debug_view<T>(item: &T, context: &Context, ui: &mut Ui)
where
    T: FTitle + FDescription + FStats + FTag + Node,
{
    ui.heading("Debug View");

    ui.collapsing("Title", |ui| {
        item.render(context).title(ui);
    });

    ui.collapsing("Tag", |ui| {
        item.render(context).tag(ui);
    });

    ui.collapsing("Card", |ui| {
        item.render(context).card(ui);
    });

    ui.collapsing("Tag Card", |ui| {
        item.render(context).tag_card(ui);
    });
}
