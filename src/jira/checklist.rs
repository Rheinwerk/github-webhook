pub struct ChecklistManipulator {
    checklist: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrStatus {
    Open,
    Merged,
    Closed,
}

impl PrStatus {
    fn prefix(&self) -> &'static str {
        match self {
            PrStatus::Open => "- ",
            PrStatus::Merged => "+ ",
            PrStatus::Closed => "x ",
        }
    }
}

impl ChecklistManipulator {
    pub fn new(checklist: &str) -> Self {
        let checklist = checklist.lines().map(ToString::to_string).collect();
        Self { checklist }
    }

    pub fn upsert_pr(&mut self, pr_url: &str, status: PrStatus) -> bool {
        if !self
            .checklist
            .iter()
            .any(|item| item.ends_with("Pull Requests"))
        {
            tracing::warn!("Missing pull request section");
            return false;
        }

        let new_entry = format!("{}{pr_url}", status.prefix());

        // Check if PR already exists with any prefix
        if let Some(pos) = self.checklist.iter().position(|item| item.ends_with(pr_url)) {
            if self.checklist[pos] == new_entry {
                tracing::debug!("Pull request already linked with same status");
                return false;
            }
            tracing::debug!("Updating pull request status");
            self.checklist[pos] = new_entry;
            return true;
        }

        self.checklist.push(new_entry);
        true
    }

    pub fn to_string(&self) -> String {
        self.checklist.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_new_pr_open() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1");

        checklist.upsert_pr("https://github.com/org/repo/pull/2", PrStatus::Open);

        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\n- https://github.com/org/repo/pull/1\n- https://github.com/org/repo/pull/2"
        );
    }

    #[test]
    fn test_add_new_pr_merged() {
        let mut checklist = ChecklistManipulator::new("## Pull Requests");

        checklist.upsert_pr("https://github.com/org/repo/pull/1", PrStatus::Merged);

        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\n+ https://github.com/org/repo/pull/1"
        );
    }

    #[test]
    fn test_add_new_pr_closed() {
        let mut checklist = ChecklistManipulator::new("## Pull Requests");

        checklist.upsert_pr("https://github.com/org/repo/pull/1", PrStatus::Closed);

        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\nx https://github.com/org/repo/pull/1"
        );
    }

    #[test]
    fn test_update_pr_status_open_to_merged() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1");

        let updated = checklist.upsert_pr("https://github.com/org/repo/pull/1", PrStatus::Merged);

        assert!(updated);
        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\n+ https://github.com/org/repo/pull/1"
        );
    }

    #[test]
    fn test_update_pr_status_open_to_closed() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1");

        let updated = checklist.upsert_pr("https://github.com/org/repo/pull/1", PrStatus::Closed);

        assert!(updated);
        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\nx https://github.com/org/repo/pull/1"
        );
    }

    #[test]
    fn test_no_update_when_same_status() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1");

        let updated = checklist.upsert_pr("https://github.com/org/repo/pull/1", PrStatus::Open);

        assert!(!updated);
    }
}
