#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LoadingState<T> {
    #[default]
    Idle,
    Loading,
    Loaded(T),
    Failed(String),
}

impl<T> LoadingState<T> {
    /// Returns true if the state is Idle
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns true if the state is Loading
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the state is Loaded
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    /// Returns a reference to the loaded data if available
    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Loaded(data) => Some(data),
            _ => None,
        }
    }

    /// Returns a mutable reference to the loaded data if available
    pub fn data_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Loaded(data) => Some(data),
            _ => None,
        }
    }
}
