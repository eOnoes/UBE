/// Application-selectable gameplay camera behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CameraMode {
    #[default]
    FirstPerson,
    ThirdPerson,
    TopDown,
}

impl CameraMode {
    /// Cycles in a stable order for the shared V-key control.
    pub fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPerson,
            Self::ThirdPerson => Self::TopDown,
            Self::TopDown => Self::FirstPerson,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_cycle_covers_every_gameplay_view_and_wraps() {
        let third = CameraMode::FirstPerson.next();
        let top_down = third.next();
        assert_eq!(third, CameraMode::ThirdPerson);
        assert_eq!(top_down, CameraMode::TopDown);
        assert_eq!(top_down.next(), CameraMode::FirstPerson);
    }
}
