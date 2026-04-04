#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionBudget {
    pub max_pending_writes: usize,
    pub max_pending_bytes: usize,
}

impl AdmissionBudget {
    pub fn new(max_pending_writes: usize, max_pending_bytes: usize) -> Self {
        Self {
            max_pending_writes,
            max_pending_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionController {
    budget: AdmissionBudget,
}

impl AdmissionController {
    pub fn new(budget: AdmissionBudget) -> Self {
        Self { budget }
    }

    pub fn try_accept(
        &self,
        pending_writes: usize,
        pending_bytes: usize,
    ) -> Result<(), &'static str> {
        if pending_writes > self.budget.max_pending_writes {
            return Err("admission thresholds exceeded");
        }

        if pending_bytes > self.budget.max_pending_bytes {
            return Err("admission thresholds exceeded");
        }

        Ok(())
    }
}
