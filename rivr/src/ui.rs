use {
    nalgebra::{Vector2},
    metrohash::{MetroHashMap},
    cassowary::{
        WeightedRelation::*,
        strength::{STRONG, REQUIRED},
        Solver, Variable,
    },

    panels::{Panel},
    Error, Resources,
};

pub struct Ui {
    pub(crate) entries: MetroHashMap<PanelId, PanelEntry>,
    next_id: u32,

    root_id: Option<PanelId>,
    target_variables: PanelVariables,
    solver: Solver,
    variable_lookup: MetroHashMap<Variable, PanelId>,

    pub resources: Resources,
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            entries: MetroHashMap::default(),
            next_id: 0,

            root_id: None,
            target_variables: PanelVariables::new(),
            solver: Solver::new(),
            variable_lookup: MetroHashMap::default(),

            resources: Resources::new(),
        }
    }

    pub fn get(&self, panel_id: PanelId) -> Option<&PanelEntry> {
        self.entries.get(&panel_id)
    }

    pub fn get_mut(&mut self, panel_id: PanelId) -> Option<&mut PanelEntry> {
        self.entries.get_mut(&panel_id)
    }

    pub fn root_id(&self) -> Result<PanelId, Error> {
        if let Some(root_id) = self.root_id {
            Ok(root_id)
        } else {
            Err(Error::NoRoot)
        }
    }

    pub fn add_panel<P: Panel>(&mut self, panel: P) -> PanelId {
        let panel_id = PanelId { id: self.next_id };
        self.next_id += 1;

        // Create the panel entry to be added later
        let entry = PanelEntry {
            panel: Box::new(panel),
            layout: PanelLayout::new(),
            variables: PanelVariables::new(),
            needs_rendering: true,
        };

        // Add its constraints
        let constraints = entry.panel.constraints(self, &entry.variables);
        self.solver.add_constraints(&constraints).unwrap();

        // Add variable lookup for resolving
        self.variable_lookup.insert(entry.variables.width, panel_id);
        self.variable_lookup.insert(entry.variables.height, panel_id);

        // Add the panel to the entries
        self.entries.insert(panel_id, entry);

        panel_id
    }

    pub fn add_root<P: Panel>(&mut self, panel: P) -> Result<PanelId, Error> {
        if self.root_id.is_some() {
            return Err(Error::RootAlreadyExists)
        }

        let panel_id = self.add_panel(panel);
        self.root_id = Some(panel_id);

        // Prepare the target variables and make sure the root's constrained
        let panel = &self.entries[&panel_id];
        self.solver.add_constraints(&[
            panel.variables.width |LE(REQUIRED)| self.target_variables.width,
            panel.variables.height |LE(REQUIRED)| self.target_variables.height,
        ]).unwrap();
        self.solver.add_edit_variable(self.target_variables.width, STRONG).unwrap();
        self.solver.add_edit_variable(self.target_variables.height, STRONG).unwrap();

        Ok(panel_id)
    }

    pub fn solve_layout(&mut self, target_size: Vector2<f32>) {
        // Make sure the entire UI is constraint to the size of the target
        self.solver.suggest_value(self.target_variables.width, target_size.x as f64).unwrap();
        self.solver.suggest_value(self.target_variables.height, target_size.y as f64).unwrap();

        // Now get all the resolved constraints and assign them
        for (variable, value) in self.solver.fetch_changes() {
            // Do not lookup the target's variables
            if *variable == self.target_variables.width ||
                *variable == self.target_variables.height {
                continue
            }

            let panel_id = self.variable_lookup[variable];
            let panel = self.entries.get_mut(&panel_id).unwrap();

            if panel.variables.width == *variable {
                panel.layout.size.x = *value as f32;
            } else {
                assert_eq!(panel.variables.height, *variable);
                panel.layout.size.y = *value as f32;
            }
        }

        // TODO: Detect unresolved variables
    }
}

pub struct PanelEntry {
    pub panel: Box<Panel>,
    pub layout: PanelLayout,
    pub variables: PanelVariables,
    pub needs_rendering: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PanelId { id: u32 }

pub struct PanelVariables {
    pub width: Variable,
    pub height: Variable,
}

impl PanelVariables {
    pub fn new() -> Self {
        PanelVariables {
            width: Variable::new(),
            height: Variable::new(),
        }
    }
}

pub struct PanelLayout {
    pub size: Vector2<f32>,
}

impl PanelLayout {
    pub fn new() -> Self {
        PanelLayout {
            size: Vector2::new(0.0, 0.0),
        }
    }
}
