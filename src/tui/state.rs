#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Splash screen with the logo.
    #[default]
    Splash,
    /// Typing a repository path.
    Inputting,
    /// Repository dashboard.
    Dashboard,
    /// Commit-message popup over the dashboard.
    Committing,
    /// Settings form ("bring your own key").
    Settings,
    /// GitHub device-flow login popup.
    Login,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Status,
    Log,
    Branches,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Status, Tab::Log, Tab::Branches];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Status => " 1 Status ",
            Tab::Log => " 2 Log ",
            Tab::Branches => " 3 Branches ",
        }
    }

    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn next(self) -> Tab {
        Tab::ALL[(self.index() + 1) % Tab::ALL.len()]
    }

    pub fn prev(self) -> Tab {
        Tab::ALL[(self.index() + Tab::ALL.len() - 1) % Tab::ALL.len()]
    }
}
