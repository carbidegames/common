use {
    cassowary::{
        WeightedRelation::*,
        strength::{WEAK, MEDIUM, REQUIRED},
        Variable, Constraint,
    },

    PanelVariables,
};

#[derive(Clone)]
pub struct PanelSize {
    pub x: AxisSize,
    pub y: AxisSize,
}

impl PanelSize {
    pub fn new(x: AxisSize, y: AxisSize) -> Self {
        PanelSize {
            x,
            y,
        }
    }

    pub fn absolute(x: f32, y: f32) -> Self {
        PanelSize {
            x: AxisSize::Absolute(x),
            y: AxisSize::Absolute(y),
        }
    }

    pub fn max() -> Self {
        PanelSize {
            x: AxisSize::Max,
            y: AxisSize::Max,
        }
    }

    pub fn min() -> Self {
        PanelSize {
            x: AxisSize::Min,
            y: AxisSize::Min,
        }
    }

    pub fn add_constraints(
        &self, constraints: &mut Vec<Constraint>, this: &PanelVariables,
    ) {
        self.x.add_constraints(constraints, this.width);
        self.y.add_constraints(constraints, this.height);
    }
}

#[derive(Copy, Clone)]
pub enum AxisSize {
    /// Tries to keep the panel to this size, but allows panel contents to overwrite it.
    Absolute(f32),
    /// Tries to make the panel as big as possible.
    Max,
    /// Tries to make the panel as small as possible.
    Min,
}

impl AxisSize {
    pub fn is_absolute(self) -> bool {
        if let AxisSize::Absolute(_) = self {
            true
        } else {
            false
        }
    }

    pub fn add_constraints(
        self,
        constraints: &mut Vec<Constraint>, axis: Variable,
    ) {
        let constraint = match self {
            AxisSize::Absolute(value) =>
                axis |EQ(MEDIUM)| value as f64,
            AxisSize::Max =>
                axis |EQ(WEAK)| 1_000_000.0,
            AxisSize::Min =>
                axis |EQ(WEAK)| 0.0,
        };

        // Must be non-negative size
        constraints.push(axis |GE(REQUIRED)| 0.0);
        // The size constraint itself
        constraints.push(constraint);
    }
}
