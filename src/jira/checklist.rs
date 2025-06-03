pub struct ChecklistManipulator {
    checklist: Vec<String>,
}

impl ChecklistManipulator {
    pub fn new(checklist: &str) -> Self {
        let checklist = checklist.lines().map(ToString::to_string).collect();
        Self { checklist }
    }

    pub fn push_pr(&mut self, pr_url: &str) -> bool {
        if !self
            .checklist
            .iter()
            .any(|item| item.ends_with("Pull Requests"))
        {
            return false;
        }

        if self.checklist.iter().any(|item| item.ends_with(pr_url)) {
            return false;
        }

        self.checklist.push(format!("- {pr_url}"));
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
    fn test_add_new_pr() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1");

        checklist.push_pr("https://github.com/org/repo/pull/2");

        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\n- https://github.com/org/repo/pull/1\n- https://github.com/org/repo/pull/2"
        );
    }

    #[test]
    fn test_add_existing_pr() {
        let mut checklist =
            ChecklistManipulator::new("## Pull Requests\n- https://github.com/org/repo/pull/1\n- https://github.com/org/repo/pull/2");

        checklist.push_pr("https://github.com/org/repo/pull/2");

        assert_eq!(
            &checklist.to_string(),
            "## Pull Requests\n- https://github.com/org/repo/pull/1\n- https://github.com/org/repo/pull/2"
        );
    }
}
