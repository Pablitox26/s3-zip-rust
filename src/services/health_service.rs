pub struct HealthService;

impl HealthService {
    pub fn new() -> Self {
        HealthService
    }

    pub fn check_health(&self) -> Result<String, &'static str> {
        Ok("Everything is working fine".to_string())
    }
}